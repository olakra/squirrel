//! The world owns all simulation state and advances it in fixed ticks.
//!
//! ## Chunks
//!
//! Every belt chain, splitter and inserter sits on a tile. Each one is counted
//! in the `ChunkGrid` for its chunk. A chunk with at least one entity is active.
//! A tick visits only active chunks, in sorted `(x, y)` order. A chunk that has
//! no entities is never visited, so it cannot change state.
//!
//! ## Tick order
//!
//! A tick runs four phases over the active chunks, in the order given by
//! `TICK_ORDER`:
//!
//! 1. `PickFromSources` (a). Inserters claim the item waiting at the exit of
//!    their source lane and lift it off the belt. An item that reached an exit
//!    during the previous tick is visible here.
//! 2. `AdvanceChains` (b). Every belt chain moves its items forward and hands
//!    items to free downstream lanes. A closed exit keeps its item in place.
//! 3. `RouteSplitters` (c). Each splitter moves at most one exit item from its
//!    input lane to an open output.
//! 4. `DropIntoTargets` (d). Inserters that have finished rotating place their
//!    held item on the rear of the target lane, if it has room.
//!
//! Within a phase, chunks are visited in sorted order. Inside a chunk, chains
//! run by index and splitters and inserters run by tile. No order depends on
//! registration order.
//!
//! The order is part of the state contract. `state_hash` covers it, so two
//! worlds that tick in different orders never share a fingerprint.
//!
//! ## Inserter timing
//!
//! An inserter takes the same number of ticks per phase as the standalone
//! `Inserter` state machine. A cycle is one claim tick, then pick, rotate, drop
//! and reset ticks. The claim happens in (a) and the drop in (d).

use std::collections::BTreeMap;

use crate::core::hash::Fnv1a;
use crate::simulation::belt::BeltChain;
use crate::simulation::chunk::{ChunkCoord, ChunkGrid, ChunkState, TileCoord};
use crate::simulation::inserter::Inserter;
use crate::simulation::wiring::{hash_tile, ChunkMembers, InserterSlot, LaneRef, SplitterSlot};

/// One phase of a world tick. See the module docs for the order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TickPhase {
    /// (a) Inserters claim items at their source exits.
    PickFromSources = 0,
    /// (b) Belt chains advance and hand items downstream.
    AdvanceChains = 1,
    /// (c) Splitters route one exit item each.
    RouteSplitters = 2,
    /// (d) Inserters drop held items onto target lanes.
    DropIntoTargets = 3,
}

/// The fixed order of tick phases. `World::step` runs these in sequence.
pub const TICK_ORDER: [TickPhase; 4] = [
    TickPhase::PickFromSources,
    TickPhase::AdvanceChains,
    TickPhase::RouteSplitters,
    TickPhase::DropIntoTargets,
];

/// Tile given to chains that `World::new` receives. They have no position of
/// their own, so they all share the origin chunk.
const ORIGIN: TileCoord = TileCoord { x: 0, y: 0 };

/// Why a placement, wiring or removal request was refused. A refused request
/// changes nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldError {
    /// No chain has this index.
    UnknownChain(usize),
    /// The chain exists but has no lane at this address.
    UnknownLane(LaneRef),
    /// A splitter's input and both outputs must sit in three distinct chains.
    SharedSplitterChain,
    /// A splitter or inserter already occupies this tile.
    TileOccupied(TileCoord),
    /// No splitter or inserter sits on this tile.
    NoEntityAt(TileCoord),
    /// A splitter or inserter still uses this chain.
    ChainInUse(usize),
}

/// The chunk that contains `tile`.
fn chunk_of(tile: TileCoord) -> ChunkCoord {
    ChunkCoord::from_tile(tile.x, tile.y)
}

/// True if any two of the three addresses name the same chain.
fn has_shared_chain(refs: [LaneRef; 3]) -> bool {
    refs[0].chain == refs[1].chain
        || refs[0].chain == refs[2].chain
        || refs[1].chain == refs[2].chain
}

/// Simulation state: belt chains, their placement, and the wiring between
/// them.
#[derive(Clone, Debug)]
pub struct World {
    tick: u64,
    chains: Vec<BeltChain>,
    /// Tile of each chain, parallel to `chains`.
    chain_tiles: Vec<TileCoord>,
    /// Entity count per chunk. Decides which chunks are active.
    grid: ChunkGrid,
    /// Entities by chunk. Only chunks with at least one entity appear here.
    members: BTreeMap<ChunkCoord, ChunkMembers>,
    /// Phase order used by `step`. Starts as `TICK_ORDER`.
    order: [TickPhase; 4],
}

impl Default for World {
    /// An empty world with no chains, no wiring and tick 0.
    fn default() -> Self {
        Self {
            tick: 0,
            chains: Vec::new(),
            chain_tiles: Vec::new(),
            grid: ChunkGrid::new(),
            members: BTreeMap::new(),
            order: TICK_ORDER,
        }
    }
}

impl World {
    /// Creates a world from chains that have no positions. Every chain is
    /// placed on the origin tile, so all of them share one chunk. Use
    /// `place_chain` to position chains.
    pub fn new(chains: Vec<BeltChain>) -> Self {
        let mut world = Self::default();
        for chain in chains {
            world.place_chain(ORIGIN, chain);
        }
        world
    }

    /// Number of fixed steps taken so far.
    #[inline]
    pub fn tick_count(&self) -> u64 {
        self.tick
    }

    /// All belt chains, in index order.
    pub fn chains(&self) -> &[BeltChain] {
        &self.chains
    }

    /// Mutable access to the belt chains. Changing a chain does not change
    /// its chunk, so the chunk grid stays valid. Do not clear a chain's sink
    /// while a splitter or inserter takes from its final lane.
    pub fn chains_mut(&mut self) -> &mut [BeltChain] {
        &mut self.chains
    }

    /// The chunk grid. A chunk is active while at least one entity sits on it.
    pub fn chunk_grid(&self) -> &ChunkGrid {
        &self.grid
    }

    /// Active chunks, in sorted `(x, y)` order.
    pub fn active_chunks(&self) -> impl Iterator<Item = ChunkCoord> + '_ {
        self.grid.active_chunks()
    }

    /// Items carried by inserters between their pickup and their drop.
    pub fn items_held_by_inserters(&self) -> usize {
        self.members.values().map(ChunkMembers::held_count).sum()
    }

    /// Adds a belt chain on `tile` and returns its index. Indices are dense
    /// and follow placement order. Chains on the same tile share a chunk.
    pub fn place_chain(&mut self, tile: TileCoord, chain: BeltChain) -> usize {
        let index = self.chains.len();
        self.chains.push(chain);
        self.chain_tiles.push(tile);
        self.grid.add_entity(tile);
        self.members
            .entry(chunk_of(tile))
            .or_default()
            .push_chain(index);
        index
    }

    /// Removes the chain at `index`. Later chains move down one index, and
    /// their wiring follows them. Refused while a splitter or inserter uses
    /// the chain.
    pub fn remove_chain(&mut self, index: usize) -> Result<(), WorldError> {
        if index >= self.chains.len() {
            return Err(WorldError::UnknownChain(index));
        }
        if self.members.values().any(|m| m.references_chain(index)) {
            return Err(WorldError::ChainInUse(index));
        }
        let tile = self.chain_tiles.remove(index);
        self.chains.remove(index);
        self.grid.remove_entity(tile);
        for members in self.members.values_mut() {
            members.shift_after_removal(index);
        }
        self.prune(chunk_of(tile));
        Ok(())
    }

    /// Adds a splitter on `tile`. It takes items from `input` and sends each
    /// one to `left` or `right`. The three lanes must sit in three distinct
    /// chains. If `input` is the final lane of its chain, that chain's exit is
    /// closed, so the splitter takes items before the chain can sink them.
    pub fn add_splitter(
        &mut self,
        tile: TileCoord,
        input: LaneRef,
        left: LaneRef,
        right: LaneRef,
    ) -> Result<(), WorldError> {
        for lane in [input, left, right] {
            self.check_lane(lane)?;
        }
        if has_shared_chain([input, left, right]) {
            return Err(WorldError::SharedSplitterChain);
        }
        let slot = SplitterSlot::new(tile, input, left, right);
        self.members
            .entry(chunk_of(tile))
            .or_default()
            .insert_splitter(slot)?;
        self.grid.add_entity(tile);
        self.close_exit_if_last(input);
        Ok(())
    }

    /// Removes the splitter on `tile`.
    pub fn remove_splitter(&mut self, tile: TileCoord) -> Result<(), WorldError> {
        self.remove_entity_at(tile, ChunkMembers::remove_splitter)
    }

    /// Adds an inserter on `tile` that takes from `source` and drops onto
    /// `target`. If `source` is the final lane of its chain, that chain's exit
    /// is closed, so the inserter takes items before the chain can sink them.
    pub fn add_inserter(
        &mut self,
        tile: TileCoord,
        inserter: Inserter,
        source: LaneRef,
        target: LaneRef,
    ) -> Result<(), WorldError> {
        self.check_lane(source)?;
        self.check_lane(target)?;
        let slot = InserterSlot::new(tile, inserter, source, target);
        self.members
            .entry(chunk_of(tile))
            .or_default()
            .insert_inserter(slot)?;
        self.grid.add_entity(tile);
        self.close_exit_if_last(source);
        Ok(())
    }

    /// Removes the inserter on `tile`. Any item it holds is lost with it.
    pub fn remove_inserter(&mut self, tile: TileCoord) -> Result<(), WorldError> {
        self.remove_entity_at(tile, ChunkMembers::remove_inserter)
    }

    /// Advances one fixed step.
    pub fn step(&mut self) {
        self.step_observed(|_| {});
    }

    /// Advances one fixed step. `on_chunk` is called for each active chunk,
    /// once per phase, in tick order. Tests use it to prove that sleeping
    /// chunks are never visited.
    fn step_observed(&mut self, mut on_chunk: impl FnMut(ChunkCoord)) {
        for phase in self.order {
            self.run_phase(phase, &mut on_chunk);
        }
        self.tick += 1;
    }

    /// Runs one phase over every active chunk, in sorted order.
    fn run_phase(&mut self, phase: TickPhase, on_chunk: &mut impl FnMut(ChunkCoord)) {
        for (coord, members) in self.members.iter_mut() {
            if self.grid.state(*coord) == ChunkState::Asleep {
                continue;
            }
            on_chunk(*coord);
            members.run(phase, &mut self.chains);
        }
    }

    /// Fingerprint of the full state, including tick count, tick order,
    /// placement and wiring.
    pub fn state_hash(&self) -> u64 {
        let mut h = Fnv1a::new();
        h.write_u64(self.tick);
        h.write_u32(self.chains.len() as u32);
        for (chain, tile) in self.chains.iter().zip(&self.chain_tiles) {
            hash_tile(*tile, &mut h);
            chain.hash_into(&mut h);
        }
        self.grid.hash_into(&mut h);
        for phase in self.order {
            h.write_u8(phase as u8);
        }
        for members in self.members.values() {
            members.hash_into(&mut h);
        }
        h.finish()
    }

    /// Refuses an address that names no lane.
    fn check_lane(&self, at: LaneRef) -> Result<(), WorldError> {
        let chain = self
            .chains
            .get(at.chain)
            .ok_or(WorldError::UnknownChain(at.chain))?;
        if at.lane < chain.lanes().len() {
            Ok(())
        } else {
            Err(WorldError::UnknownLane(at))
        }
    }

    /// Closes the exit of a chain whose final lane is taken by a consumer.
    /// Without this, the chain would sink the item before the consumer runs.
    fn close_exit_if_last(&mut self, at: LaneRef) {
        let chain = &mut self.chains[at.chain];
        if at.lane + 1 == chain.lanes().len() {
            chain.set_sink_closed(true);
        }
    }

    /// Removes one splitter or inserter, found by tile, and clears its chunk
    /// count. `remove` does the bucket-level removal.
    fn remove_entity_at(
        &mut self,
        tile: TileCoord,
        remove: impl FnOnce(&mut ChunkMembers, TileCoord) -> bool,
    ) -> Result<(), WorldError> {
        let coord = chunk_of(tile);
        let removed = self
            .members
            .get_mut(&coord)
            .is_some_and(|m| remove(m, tile));
        if !removed {
            return Err(WorldError::NoEntityAt(tile));
        }
        self.grid.remove_entity(tile);
        self.prune(coord);
        Ok(())
    }

    /// Drops the bucket for `coord` once it holds nothing, so only chunks with
    /// entities stay in the map.
    fn prune(&mut self, coord: ChunkCoord) {
        if self.members.get(&coord).is_some_and(ChunkMembers::is_empty) {
            self.members.remove(&coord);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ids::ItemId;
    use crate::simulation::belt::BeltLane;

    /// Pick 2, rotate 3, drop 1, reset 2. A full cycle is 9 ticks.
    const PICK: u32 = 2;
    const ROTATE: u32 = 3;
    const DROP: u32 = 1;
    const RESET: u32 = 2;
    const CYCLE: u64 = 1 + (PICK + ROTATE + DROP + RESET) as u64;

    fn at(x: i32, y: i32) -> TileCoord {
        TileCoord { x, y }
    }

    fn origin_chunk() -> ChunkCoord {
        ChunkCoord { x: 0, y: 0 }
    }

    fn lane() -> BeltLane {
        BeltLane::with_capacity(40, 2, 16)
    }

    fn lane_ref(chain: usize, lane: usize) -> LaneRef {
        LaneRef { chain, lane }
    }

    fn inserter() -> Inserter {
        Inserter::new(PICK, ROTATE, DROP, RESET)
    }

    fn single_lane_chain(lane: BeltLane) -> BeltChain {
        BeltChain::new(vec![lane])
    }

    /// Offers one item to chain `chain` if its feed end has room.
    fn feed(w: &mut World, chain: usize, item: u16) -> bool {
        w.chains_mut()[chain].try_feed(ItemId(item))
    }

    /// Items on belts, sunk into sinks, and held by inserters. Every item
    /// fed into the world is in exactly one of these.
    fn items_accounted_for(w: &World) -> u64 {
        let on_belts: usize = w
            .chains()
            .iter()
            .flat_map(|c| c.lanes())
            .map(BeltLane::len)
            .sum();
        let sunk: u64 = w.chains().iter().map(BeltChain::sunk).sum();
        on_belts as u64 + sunk + w.items_held_by_inserters() as u64
    }

    /// Source (chain 0) -> two inserters -> middle (chain 1) -> splitter ->
    /// left (chain 2) and right (chain 3). Chains 0 and 1 share the origin
    /// chunk, chain 2 sits in chunk (1, 0) with the splitter, and chain 3 sits
    /// in chunk (1, 1). The two inserters are registered in either order.
    fn mixed_world(reversed: bool) -> World {
        const SOURCE: usize = 0;
        const MIDDLE: usize = 1;
        const LEFT: usize = 2;
        const RIGHT: usize = 3;

        let mut w = World::default();
        w.place_chain(at(0, 0), single_lane_chain(lane()));
        w.place_chain(at(0, 0), single_lane_chain(lane()));
        w.place_chain(at(32, 0), single_lane_chain(lane()));
        w.place_chain(at(32, 32), single_lane_chain(lane()));

        let mut feeds = [
            (at(1, 0), lane_ref(SOURCE, 0), lane_ref(MIDDLE, 0)),
            (at(2, 0), lane_ref(SOURCE, 0), lane_ref(MIDDLE, 0)),
        ];
        if reversed {
            feeds.reverse();
        }
        for (tile, source, target) in feeds {
            w.add_inserter(tile, inserter(), source, target).unwrap();
        }
        w.add_splitter(
            at(40, 0),
            lane_ref(MIDDLE, 0),
            lane_ref(LEFT, 0),
            lane_ref(RIGHT, 0),
        )
        .unwrap();
        w
    }

    /// Feeds the source chain of `mixed_world` every third tick, then steps.
    /// Returns the number of items the source accepted.
    fn run_mixed(w: &mut World, ticks: u64) -> u64 {
        let mut fed = 0;
        for t in 0..ticks {
            if t % 3 == 0 && feed(w, 0, (t % 7) as u16) {
                fed += 1;
            }
            w.step();
        }
        fed
    }

    #[test]
    fn determinism_1000_ticks_same_hash_across_runs() {
        let a = scenario();
        let b = scenario();
        assert_eq!(a.tick_count(), 1_000);
        assert_eq!(a.state_hash(), b.state_hash());
    }

    /// A fixed layout, identical on every run. The feed pattern is a pure
    /// function of the tick, so there is no hidden input.
    fn scenario() -> World {
        let chain = BeltChain::new(vec![
            BeltLane::with_capacity(60, 2, 128),
            BeltLane::with_capacity(60, 3, 128),
            BeltLane::with_capacity(90, 1, 128),
        ]);
        let mut w = World::new(vec![chain]);
        for t in 0..1_000u64 {
            if t % 5 == 0 {
                let _ = w.chains_mut()[0].try_feed(ItemId((t % 7) as u16));
            }
            w.step();
        }
        w
    }

    #[test]
    fn hash_detects_a_single_changed_item() {
        let a = scenario();
        let mut b = scenario();
        // Items are private, so change the state through a fresh run that
        // feeds one extra item at a different tick.
        let _ = b.chains_mut()[0].try_feed(ItemId(9));
        assert_ne!(a.state_hash(), b.state_hash());
    }

    #[test]
    fn tick_order_is_pick_advance_route_drop() {
        assert_eq!(
            TICK_ORDER,
            [
                TickPhase::PickFromSources,
                TickPhase::AdvanceChains,
                TickPhase::RouteSplitters,
                TickPhase::DropIntoTargets,
            ]
        );
    }

    #[test]
    fn only_active_chunks_are_ticked() {
        let far = ChunkCoord { x: 3, y: 3 };
        let mut w = World::new(vec![single_lane_chain(lane())]);
        let far_index = w.place_chain(at(100, 100), single_lane_chain(lane()));
        assert_eq!(
            w.active_chunks().collect::<Vec<_>>(),
            vec![origin_chunk(), far]
        );

        w.remove_chain(far_index).unwrap();
        assert_eq!(w.active_chunks().collect::<Vec<_>>(), vec![origin_chunk()]);
        assert_eq!(w.chunk_grid().state(far), ChunkState::Asleep);

        let mut visits = Vec::new();
        for _ in 0..10 {
            w.step_observed(|coord| visits.push(coord));
        }
        assert_eq!(
            visits.len(),
            TICK_ORDER.len() * 10,
            "the one active chunk is visited once per phase"
        );
        assert!(
            visits.iter().all(|&c| c == origin_chunk()),
            "a sleeping chunk must never be visited"
        );
    }

    #[test]
    fn removing_an_inserter_frees_its_tile() {
        let mut w = World::default();
        let s = w.place_chain(at(0, 0), single_lane_chain(lane()));
        let t = w.place_chain(at(0, 0), single_lane_chain(lane()));
        w.add_inserter(at(1, 0), inserter(), lane_ref(s, 0), lane_ref(t, 0))
            .unwrap();
        assert_eq!(w.remove_inserter(at(1, 0)), Ok(()));
        assert_eq!(
            w.remove_inserter(at(1, 0)),
            Err(WorldError::NoEntityAt(at(1, 0)))
        );
        assert!(w
            .add_inserter(at(1, 0), inserter(), lane_ref(s, 0), lane_ref(t, 0))
            .is_ok());
    }

    #[test]
    fn remove_chain_is_refused_while_wired() {
        let mut w = World::default();
        let s = w.place_chain(at(0, 0), single_lane_chain(lane()));
        let t = w.place_chain(at(0, 0), single_lane_chain(lane()));
        w.add_inserter(at(1, 0), inserter(), lane_ref(s, 0), lane_ref(t, 0))
            .unwrap();
        assert_eq!(w.remove_chain(s), Err(WorldError::ChainInUse(s)));
        assert_eq!(w.chains().len(), 2, "a refused removal changes nothing");
    }

    #[test]
    fn removing_a_chain_renumbers_the_wiring_after_it() {
        let mut w = World::default();
        w.place_chain(at(0, 0), single_lane_chain(lane()));
        let s = w.place_chain(at(0, 0), single_lane_chain(lane()));
        let t = w.place_chain(at(0, 0), single_lane_chain(lane()));
        w.add_inserter(at(1, 0), inserter(), lane_ref(s, 0), lane_ref(t, 0))
            .unwrap();
        w.remove_chain(0).unwrap();
        assert_eq!(w.chains().len(), 2);

        // The source is now chain 0 and the target chain 1. Items must still
        // flow from one to the other.
        for t in 0..200u16 {
            feed(&mut w, 0, t % 7);
            w.step();
        }
        assert!(
            w.chains()[1].sunk() > 0,
            "moved items reach the renumbered target"
        );
    }

    #[test]
    fn add_splitter_rejects_bad_wiring() {
        let mut w = World::default();
        let a = w.place_chain(at(0, 0), single_lane_chain(lane()));
        let b = w.place_chain(at(0, 0), single_lane_chain(lane()));
        let c = w.place_chain(at(0, 0), single_lane_chain(lane()));
        assert_eq!(
            w.add_splitter(at(5, 5), lane_ref(a, 0), lane_ref(a, 0), lane_ref(b, 0)),
            Err(WorldError::SharedSplitterChain)
        );
        assert_eq!(
            w.add_splitter(at(5, 5), lane_ref(a, 0), lane_ref(b, 7), lane_ref(c, 0)),
            Err(WorldError::UnknownLane(lane_ref(b, 7)))
        );
        assert_eq!(
            w.add_splitter(at(5, 5), lane_ref(a, 0), lane_ref(9, 0), lane_ref(c, 0)),
            Err(WorldError::UnknownChain(9))
        );
        w.add_splitter(at(5, 5), lane_ref(a, 0), lane_ref(b, 0), lane_ref(c, 0))
            .unwrap();
        assert_eq!(
            w.add_inserter(at(5, 5), inserter(), lane_ref(a, 0), lane_ref(b, 0)),
            Err(WorldError::TileOccupied(at(5, 5))),
            "a splitter and an inserter cannot share a tile"
        );
    }

    #[test]
    fn inserter_holds_its_item_while_target_is_full() {
        let mut w = World::default();
        let source = w.place_chain(at(0, 0), single_lane_chain(lane()));
        // Length 12 with spacing 4 holds three items, at offsets 8, 4 and 0.
        let target = w.place_chain(
            at(0, 0),
            single_lane_chain(BeltLane::with_capacity(12, 2, 16)),
        );
        w.add_inserter(
            at(1, 0),
            inserter(),
            lane_ref(source, 0),
            lane_ref(target, 0),
        )
        .unwrap();
        w.chains_mut()[target].set_sink_closed(true);

        let mut fed = 0;
        for t in 0..300u16 {
            if feed(&mut w, source, t % 7) {
                fed += 1;
            }
            w.step();
        }
        assert_eq!(w.chains()[target].lanes()[0].len(), 3, "target is full");
        assert_eq!(
            w.items_held_by_inserters(),
            1,
            "the inserter holds one item"
        );
        assert_eq!(items_accounted_for(&w), fed);

        w.chains_mut()[target].set_sink_closed(false);
        for t in 0..300u16 {
            if feed(&mut w, source, t % 7) {
                fed += 1;
            }
            w.step();
        }
        assert_eq!(items_accounted_for(&w), fed, "no item is lost on resume");
        assert!(
            w.chains()[target].sunk() > 0,
            "held items drain once the sink opens"
        );
    }

    #[test]
    fn drops_arrive_once_per_standalone_cycle() {
        // A fast, packed source keeps an item at its exit, so the inserter never
        // waits for stock. The target sink is open, so each drop only adds to
        // the target's total.
        let mut w = World::default();
        let source = w.place_chain(
            at(0, 0),
            single_lane_chain(BeltLane::with_capacity(40, 4, 16)),
        );
        let target = w.place_chain(
            at(0, 0),
            single_lane_chain(BeltLane::with_capacity(40, 4, 16)),
        );
        w.add_inserter(
            at(1, 0),
            inserter(),
            lane_ref(source, 0),
            lane_ref(target, 0),
        )
        .unwrap();

        let delivered = |w: &World| {
            let c = &w.chains()[target];
            c.sunk() + c.lanes()[0].len() as u64
        };
        let mut drops = Vec::new();
        let mut last = 0;
        for t in 0..200u64 {
            feed(&mut w, source, 1);
            w.step();
            if delivered(&w) > last {
                drops.push(t);
                last = delivered(&w);
            }
        }
        let gaps: Vec<u64> = drops.windows(2).map(|p| p[1] - p[0]).collect();
        assert!(gaps.len() > 5, "expected many drops, got {drops:?}");
        assert!(
            gaps.iter().all(|&g| g == CYCLE),
            "every gap must match the standalone cycle of {CYCLE}: {gaps:?}"
        );
    }

    #[test]
    fn mixed_layout_is_deterministic_over_2000_ticks() {
        let mut a = mixed_world(false);
        run_mixed(&mut a, 2_000);
        let mut b = mixed_world(false);
        run_mixed(&mut b, 2_000);
        assert_eq!(a.state_hash(), b.state_hash(), "same layout, same hash");

        let mut reversed = mixed_world(true);
        run_mixed(&mut reversed, 2_000);
        assert_eq!(
            a.state_hash(),
            reversed.state_hash(),
            "inserter registration order must not change the result"
        );
    }

    #[test]
    fn mixed_layout_conserves_items_and_feeds_both_sinks() {
        let mut w = mixed_world(false);
        let fed = run_mixed(&mut w, 2_000);
        assert_eq!(
            items_accounted_for(&w),
            fed,
            "every fed item is accounted for"
        );

        let left = w.chains()[2].sunk();
        let right = w.chains()[3].sunk();
        assert!(left > 0 && right > 0, "both splitter outputs deliver items");
        assert!(
            left.abs_diff(right) <= 2,
            "a splitter with open outputs alternates: left={left} right={right}"
        );
    }

    #[test]
    fn changing_tick_order_changes_state_hash() {
        let mut base = mixed_world(false);
        run_mixed(&mut base, 2_000);

        let mut swapped = mixed_world(false);
        swapped.order = [
            TickPhase::AdvanceChains,
            TickPhase::PickFromSources,
            TickPhase::RouteSplitters,
            TickPhase::DropIntoTargets,
        ];
        run_mixed(&mut swapped, 2_000);

        assert_ne!(
            base.state_hash(),
            swapped.state_hash(),
            "the tick order is part of the state contract"
        );
    }
}
