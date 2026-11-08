//! Splitter and inserter records, wired to belt lanes by reference.
//!
//! Belt chains own their lanes. A splitter or inserter owns no lanes. It stores
//! a `LaneRef` (chain index and lane index) for each lane it touches, and the
//! world resolves that reference when a tick phase runs.
//!
//! Records are grouped by chunk in `ChunkMembers`. Splitters and inserters are
//! kept sorted by tile, so the order in which they run depends on where they
//! sit, not on the order in which they were registered.
//!
//! An inserter's `Inserter` state machine runs in two gated halves. The pick
//! half runs in phase (a) and only while the inserter is searching or picking
//! up. The drop half runs in phase (d) and only while it is rotating, dropping
//! or resetting. Each phase therefore counts down exactly once per tick, and
//! the source and target stock are only ever touched by their own half.

use crate::core::hash::Fnv1a;
use crate::core::ids::ItemId;
use crate::simulation::belt::{BeltChain, BeltLane};
use crate::simulation::chunk::TileCoord;
use crate::simulation::inserter::{Inserter, Inventory, Phase};
use crate::simulation::splitter::Splitter;
use crate::simulation::world::{TickPhase, WorldError};

/// Address of one lane: the chain it belongs to, and its index in that chain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LaneRef {
    /// Index into `World::chains`.
    pub chain: usize,
    /// Index into `BeltChain::lanes`. Index 0 is the feed end.
    pub lane: usize,
}

impl LaneRef {
    /// Feeds the address into `h`.
    pub fn hash_into(&self, h: &mut Fnv1a) {
        h.write_u32(self.chain as u32);
        h.write_u32(self.lane as u32);
    }

    /// Renumbers the reference after the chain at `removed` is deleted.
    /// Chains after it move down one index.
    fn shift_after_removal(&mut self, removed: usize) {
        if self.chain > removed {
            self.chain -= 1;
        }
    }
}

/// Feeds a tile position into `h`. Negative coordinates keep their bits, so
/// they still hash differently from positive ones.
pub fn hash_tile(tile: TileCoord, h: &mut Fnv1a) {
    h.write_u32(tile.x as u32);
    h.write_u32(tile.y as u32);
}

/// Borrows the lane at `at` mutably.
fn lane_mut(chains: &mut [BeltChain], at: LaneRef) -> &mut BeltLane {
    &mut chains[at.chain].lanes_mut()[at.lane]
}

/// Borrows three lanes mutably at once. Their chains must be distinct, which
/// `World::add_splitter` checks when the splitter is registered.
fn three_lanes_mut(chains: &mut [BeltChain], refs: [LaneRef; 3]) -> [&mut BeltLane; 3] {
    let [a, b, c] = chains
        .get_disjoint_mut(refs.map(|r| r.chain))
        .expect("splitter lanes live in distinct chains");
    [
        &mut a.lanes_mut()[refs[0].lane],
        &mut b.lanes_mut()[refs[1].lane],
        &mut c.lanes_mut()[refs[2].lane],
    ]
}

/// Anything placed on a tile. Buckets keep their entries sorted by tile.
trait Placed {
    fn tile(&self) -> TileCoord;
}

/// Sort key for a tile: `x`, then `y`.
fn tile_key(tile: TileCoord) -> (i32, i32) {
    (tile.x, tile.y)
}

/// Inserts `item` in tile order. Refuses a tile that is already taken.
fn insert_sorted<T: Placed>(items: &mut Vec<T>, item: T) -> Result<(), WorldError> {
    let tile = item.tile();
    match items.binary_search_by_key(&tile_key(tile), |other| tile_key(other.tile())) {
        Ok(_) => Err(WorldError::TileOccupied(tile)),
        Err(pos) => {
            items.insert(pos, item);
            Ok(())
        }
    }
}

/// Removes the entry on `tile`. Returns `false` if there is none.
fn remove_sorted<T: Placed>(items: &mut Vec<T>, tile: TileCoord) -> bool {
    match items.binary_search_by_key(&tile_key(tile), |other| tile_key(other.tile())) {
        Ok(pos) => {
            items.remove(pos);
            true
        }
        Err(_) => false,
    }
}

/// True if any entry in `items` sits on `tile`.
fn has_tile<T: Placed>(items: &[T], tile: TileCoord) -> bool {
    items
        .binary_search_by_key(&tile_key(tile), |other| tile_key(other.tile()))
        .is_ok()
}

/// A splitter on one tile, routing from one input lane to two outputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SplitterSlot {
    tile: TileCoord,
    splitter: Splitter,
    input: LaneRef,
    left: LaneRef,
    right: LaneRef,
}

impl SplitterSlot {
    /// A splitter on `tile` that takes from `input` and feeds `left` or `right`.
    pub fn new(tile: TileCoord, input: LaneRef, left: LaneRef, right: LaneRef) -> Self {
        Self {
            tile,
            splitter: Splitter::new(),
            input,
            left,
            right,
        }
    }

    /// Phase (c): moves at most one item from the input exit to an open output.
    fn route(&mut self, chains: &mut [BeltChain]) {
        let [input, left, right] = three_lanes_mut(chains, self.lane_refs());
        self.splitter.route_front(input, left, right);
    }

    /// The three lanes this splitter touches, input first.
    fn lane_refs(&self) -> [LaneRef; 3] {
        [self.input, self.left, self.right]
    }

    /// Mutable references to the three lane addresses, for renumbering.
    fn lane_refs_mut(&mut self) -> [&mut LaneRef; 3] {
        [&mut self.input, &mut self.left, &mut self.right]
    }

    fn hash_into(&self, h: &mut Fnv1a) {
        hash_tile(self.tile, h);
        self.splitter.hash_into(h);
        for r in self.lane_refs() {
            r.hash_into(h);
        }
    }
}

impl Placed for SplitterSlot {
    fn tile(&self) -> TileCoord {
        self.tile
    }
}

/// An inserter on one tile, moving items from a source lane to a target lane.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InserterSlot {
    tile: TileCoord,
    inserter: Inserter,
    source: LaneRef,
    target: LaneRef,
    /// The item lifted off the source exit. Carried until it is dropped.
    held: Option<ItemId>,
    /// Set when a pickup completes in phase (a). The rotation then starts on
    /// the next tick, the same as in the standalone state machine, so the
    /// drop half skips this tick.
    pickup_just_finished: bool,
}

impl InserterSlot {
    /// An idle inserter on `tile` that moves items from `source` to `target`.
    pub fn new(tile: TileCoord, inserter: Inserter, source: LaneRef, target: LaneRef) -> Self {
        Self {
            tile,
            inserter,
            source,
            target,
            held: None,
            pickup_just_finished: false,
        }
    }

    /// Phase (a): claims the item waiting at the source exit, or advances a
    /// pickup already in progress.
    fn pick_from_source(&mut self, chains: &mut [BeltChain]) {
        match self.inserter.phase() {
            Phase::SearchingSource => self.claim_exit_item(chains),
            Phase::PickingUp => self.run_pickup_timer(),
            _ => {}
        }
    }

    /// Lifts the exit item off the source lane if one is waiting. The item
    /// leaves the belt when it is claimed and is carried in `held` from then
    /// on.
    fn claim_exit_item(&mut self, chains: &mut [BeltChain]) {
        let source = lane_mut(chains, self.source);
        let mut offer = Inventory::new(u32::from(source.front_ready()), 1);
        self.inserter.tick(&mut offer, &mut unused_stock());
        if offer.reserved() == 1 {
            self.held = source.pop_front_if_ready();
            debug_assert!(self.held.is_some(), "a reserved exit item is ready");
        }
    }

    /// Advances the pickup timer. The item is already held, so the stock
    /// passed here is a token that lets the state machine finish its pickup.
    fn run_pickup_timer(&mut self) {
        let mut token = Inventory::new(1, 1);
        let reserved = token.reserve_one();
        debug_assert!(reserved, "a fresh token inventory has a free item");
        self.inserter.tick(&mut token, &mut unused_stock());
        self.pickup_just_finished = self.inserter.phase() == Phase::RotatingToTarget;
    }

    /// Phase (d): advances rotation, drop and reset, and drops the held item
    /// onto the target if the target lane has room.
    fn drop_into_target(&mut self, chains: &mut [BeltChain]) {
        if std::mem::take(&mut self.pickup_just_finished) {
            return;
        }
        match self.inserter.phase() {
            Phase::RotatingToTarget | Phase::Resetting => {
                self.inserter.tick(&mut unused_stock(), &mut unused_stock());
            }
            Phase::Dropping => self.drop_held(chains),
            _ => {}
        }
    }

    /// Offers the target rear one slot. The state machine drops only when
    /// its timer has run out and the slot is open. The held item is then
    /// placed on the lane.
    fn drop_held(&mut self, chains: &mut [BeltChain]) {
        let target = lane_mut(chains, self.target);
        let mut slot = Inventory::new(0, u32::from(target.can_accept_rear()));
        self.inserter.tick(&mut unused_stock(), &mut slot);
        if slot.count() == 1 {
            let item = self.held.take().expect("a dropping inserter holds an item");
            let placed = target.try_push_rear(item);
            debug_assert!(placed, "can_accept_rear was true");
        }
    }

    /// The source and target addresses, for reference checks.
    fn lane_refs(&self) -> [LaneRef; 2] {
        [self.source, self.target]
    }

    /// Mutable references to the two lane addresses, for renumbering.
    fn lane_refs_mut(&mut self) -> [&mut LaneRef; 2] {
        [&mut self.source, &mut self.target]
    }

    fn hash_into(&self, h: &mut Fnv1a) {
        hash_tile(self.tile, h);
        self.inserter.hash_into(h);
        for r in self.lane_refs() {
            r.hash_into(h);
        }
        match self.held {
            Some(item) => {
                h.write_u8(1);
                h.write_u16(item.0);
            }
            None => h.write_u8(0),
        }
        h.write_u8(u8::from(self.pickup_just_finished));
    }
}

impl Placed for InserterSlot {
    fn tile(&self) -> TileCoord {
        self.tile
    }
}

/// A stock with nothing in it. Passed to a state-machine call whose phase
/// never reads that side, so the empty stock is never touched.
fn unused_stock() -> Inventory {
    Inventory::new(0, 0)
}

/// The splitters, inserters and chain indices that sit on one chunk.
///
/// Chain indices are ascending. Splitters and inserters are sorted by tile.
/// Every bucket is non-empty, or it is removed from the world.
#[derive(Clone, Debug, Default)]
pub struct ChunkMembers {
    chains: Vec<usize>,
    splitters: Vec<SplitterSlot>,
    inserters: Vec<InserterSlot>,
}

impl ChunkMembers {
    /// True when the bucket holds nothing.
    pub fn is_empty(&self) -> bool {
        self.chains.is_empty() && self.splitters.is_empty() && self.inserters.is_empty()
    }

    /// Records that the chain at `index` sits on this chunk. Indices must
    /// arrive in ascending order, which `World::place_chain` guarantees.
    pub fn push_chain(&mut self, index: usize) {
        debug_assert!(self.chains.last().is_none_or(|&last| last < index));
        self.chains.push(index);
    }

    /// Adds a splitter. Refuses a tile that holds a splitter or an inserter.
    pub fn insert_splitter(&mut self, slot: SplitterSlot) -> Result<(), WorldError> {
        self.check_free(slot.tile())?;
        insert_sorted(&mut self.splitters, slot)
    }

    /// Adds an inserter. Refuses a tile that holds a splitter or an inserter.
    pub fn insert_inserter(&mut self, slot: InserterSlot) -> Result<(), WorldError> {
        self.check_free(slot.tile())?;
        insert_sorted(&mut self.inserters, slot)
    }

    /// Removes the splitter on `tile`. Returns `false` if there is none.
    pub fn remove_splitter(&mut self, tile: TileCoord) -> bool {
        remove_sorted(&mut self.splitters, tile)
    }

    /// Removes the inserter on `tile`. Returns `false` if there is none.
    pub fn remove_inserter(&mut self, tile: TileCoord) -> bool {
        remove_sorted(&mut self.inserters, tile)
    }

    /// Number of inserters on this chunk that hold an item.
    pub fn held_count(&self) -> usize {
        self.inserters.iter().filter(|s| s.held.is_some()).count()
    }

    /// True if a splitter or inserter in this bucket uses `index`.
    pub fn references_chain(&self, index: usize) -> bool {
        self.splitters
            .iter()
            .any(|s| s.lane_refs().iter().any(|r| r.chain == index))
            || self
                .inserters
                .iter()
                .any(|s| s.lane_refs().iter().any(|r| r.chain == index))
    }

    /// Forgets the chain at `removed` and renumbers every later chain. Called
    /// on every bucket when a chain is deleted. Only chains with no wiring
    /// reach here, so nothing refers to `removed`.
    pub fn shift_after_removal(&mut self, removed: usize) {
        self.chains.retain(|&i| i != removed);
        for i in &mut self.chains {
            if *i > removed {
                *i -= 1;
            }
        }
        for slot in &mut self.splitters {
            for r in slot.lane_refs_mut() {
                r.shift_after_removal(removed);
            }
        }
        for slot in &mut self.inserters {
            for r in slot.lane_refs_mut() {
                r.shift_after_removal(removed);
            }
        }
    }

    /// Runs one phase over this chunk's entities.
    pub fn run(&mut self, phase: TickPhase, chains: &mut [BeltChain]) {
        match phase {
            TickPhase::PickFromSources => {
                for slot in &mut self.inserters {
                    slot.pick_from_source(chains);
                }
            }
            TickPhase::AdvanceChains => {
                for &index in &self.chains {
                    chains[index].tick();
                }
            }
            TickPhase::RouteSplitters => {
                for slot in &mut self.splitters {
                    slot.route(chains);
                }
            }
            TickPhase::DropIntoTargets => {
                for slot in &mut self.inserters {
                    slot.drop_into_target(chains);
                }
            }
        }
    }

    /// Hashes the bucket. Entries are already sorted, so the hash does not
    /// depend on registration order.
    pub fn hash_into(&self, h: &mut Fnv1a) {
        h.write_u32(self.chains.len() as u32);
        for &index in &self.chains {
            h.write_u32(index as u32);
        }
        h.write_u32(self.splitters.len() as u32);
        for slot in &self.splitters {
            slot.hash_into(h);
        }
        h.write_u32(self.inserters.len() as u32);
        for slot in &self.inserters {
            slot.hash_into(h);
        }
    }

    /// Refuses a tile that already holds a splitter or an inserter.
    fn check_free(&self, tile: TileCoord) -> Result<(), WorldError> {
        if has_tile(&self.splitters, tile) || has_tile(&self.inserters, tile) {
            Err(WorldError::TileOccupied(tile))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::hash::Fnv1a;

    fn at(x: i32, y: i32) -> TileCoord {
        TileCoord { x, y }
    }

    fn lane_at(chain: usize) -> LaneRef {
        LaneRef { chain, lane: 0 }
    }

    fn splitter_on(x: i32) -> SplitterSlot {
        SplitterSlot::new(at(x, 0), lane_at(0), lane_at(1), lane_at(2))
    }

    fn hash_of(bucket: &ChunkMembers) -> u64 {
        let mut h = Fnv1a::new();
        bucket.hash_into(&mut h);
        h.finish()
    }

    #[test]
    fn lane_ref_renumbers_only_chains_after_the_removed_one() {
        let mut after = LaneRef { chain: 3, lane: 1 };
        after.shift_after_removal(1);
        assert_eq!(after, LaneRef { chain: 2, lane: 1 });

        let mut before = LaneRef { chain: 0, lane: 0 };
        before.shift_after_removal(1);
        assert_eq!(before, LaneRef { chain: 0, lane: 0 });
    }

    #[test]
    fn bucket_hash_ignores_registration_order() {
        let mut forward = ChunkMembers::default();
        for x in [1, 2, 3] {
            forward.insert_splitter(splitter_on(x)).unwrap();
        }
        let mut backward = ChunkMembers::default();
        for x in [3, 1, 2] {
            backward.insert_splitter(splitter_on(x)).unwrap();
        }
        assert_eq!(hash_of(&forward), hash_of(&backward));
    }

    #[test]
    fn bucket_refuses_a_tile_held_by_any_entity() {
        let mut bucket = ChunkMembers::default();
        bucket.insert_splitter(splitter_on(5)).unwrap();
        assert_eq!(
            bucket.insert_splitter(splitter_on(5)),
            Err(WorldError::TileOccupied(at(5, 0)))
        );
        let inserter =
            InserterSlot::new(at(5, 0), Inserter::new(1, 1, 1, 1), lane_at(0), lane_at(1));
        assert_eq!(
            bucket.insert_inserter(inserter),
            Err(WorldError::TileOccupied(at(5, 0)))
        );
        assert!(bucket.remove_splitter(at(5, 0)));
        assert!(bucket.insert_inserter(inserter).is_ok());
    }
}
