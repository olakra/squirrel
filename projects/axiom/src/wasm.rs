//! C-ABI export layer for the browser build.
//!
//! This module is a thin shim. It converts C-compatible types and forwards
//! to `World`. It contains no simulation logic. It is hand-written because
//! `wasm-bindgen` cannot be downloaded in the build environment.
//!
//! Calling convention: JavaScript passes and receives only `u32`. The `u64`
//! state hash is returned as two halves (`axiom_state_hash_lo` and
//! `axiom_state_hash_hi`), because JS cannot read a `u64` directly.
//!
//! Storage: a `thread_local!` `RefCell<Option<World>>`, not a `static mut`.
//! The world is owned by whichever thread calls `axiom_init`. Access goes
//! through `RefCell`, so the layer needs no `unsafe` and no `Sync` impl. A
//! `static mut` would share state across threads with no data-race
//! protection. With atomics enabled, each Web Worker gets its own world, and
//! every call must come from the thread that initialised it.
//!
//! Failure behaviour: calls made before a successful `axiom_init`, or after
//! an init with invalid arguments, are no-ops that return 0. `axiom_feed`
//! also returns 0 for an item id above `u16::MAX`. Counters that exceed
//! `u32::MAX` saturate.
//!
//! The FFI functions carry no `unsafe` and no pointer arguments, so no
//! `# Safety` sections are needed. The module compiles natively only for
//! `cargo test`, so the helpers below can be unit tested. The FFI functions
//! are exercised by a browser test, which is a follow-up.

use std::cell::RefCell;

use crate::core::ids::ItemId;
use crate::simulation::belt::{BeltChain, BeltLane, ITEM_SPACING};
use crate::simulation::world::World;

thread_local! {
    static WORLD: RefCell<Option<World>> = const { RefCell::new(None) };
    /// Items copied out by the last `axiom_snapshot_items` call. The per-item
    /// getters read from here, so one frame costs one walk of the world.
    static SNAPSHOT: RefCell<Vec<ItemSnapshot>> = const { RefCell::new(Vec::new()) };
}

/// One item as the renderer sees it: its id and where it sits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemSnapshot {
    pub item: u16,
    /// Index of the lane across the whole world, in chain order.
    pub lane: u32,
    /// Distance from the start of that lane.
    pub offset: u32,
}

/// Collects every item on every lane. Lanes are numbered in chain order, and
/// items are listed rear to front within each lane.
fn collect_items(chains: &[BeltChain]) -> Vec<ItemSnapshot> {
    let mut out = Vec::new();
    let mut lane_index: u32 = 0;
    for chain in chains {
        for lane in chain.lanes() {
            for it in lane.items() {
                out.push(ItemSnapshot {
                    item: it.item.0,
                    lane: lane_index,
                    offset: it.offset,
                });
            }
            lane_index = lane_index.saturating_add(1);
        }
    }
    out
}

/// Splits a `u64` into `(low, high)` 32-bit halves.
#[inline]
fn split_u64(v: u64) -> (u32, u32) {
    (v as u32, (v >> 32) as u32)
}

/// Converts a `u64` counter to `u32`, saturating at `u32::MAX`.
#[inline]
fn saturate_u32(v: u64) -> u32 {
    u32::try_from(v).unwrap_or(u32::MAX)
}

/// Maximum items a lane can hold at minimum spacing. Used to size the
/// reserved buffer so pushes never reallocate.
#[inline]
fn lane_capacity(lane_length: u32) -> usize {
    (lane_length / ITEM_SPACING) as usize
}

/// True when the arguments describe a lane that the simulation accepts.
#[inline]
fn is_valid_layout(lane_count: u32, lane_length: u32, lane_speed: u32) -> bool {
    lane_count > 0 && lane_length > ITEM_SPACING && lane_speed > 0
}

/// Builds a world holding one chain of `lane_count` identical lanes, or
/// `None` if the layout is invalid.
fn build_world(lane_count: u32, lane_length: u32, lane_speed: u32) -> Option<World> {
    if !is_valid_layout(lane_count, lane_length, lane_speed) {
        return None;
    }
    let capacity = lane_capacity(lane_length);
    let lanes: Vec<BeltLane> = (0..lane_count)
        .map(|_| BeltLane::with_capacity(lane_length, lane_speed, capacity))
        .collect();
    Some(World::new(vec![BeltChain::new(lanes)]))
}

/// Runs `f` on the world if one exists. Returns `None` otherwise.
fn with_world<R>(f: impl FnOnce(&mut World) -> R) -> Option<R> {
    WORLD.with(|cell| {
        let mut slot = cell.try_borrow_mut().ok()?;
        slot.as_mut().map(f)
    })
}

/// Creates (or replaces) the world: a linear chain of `lane_count` lanes,
/// each `lane_length` units long and moving `lane_speed` units per tick.
///
/// Invalid arguments (zero lanes, zero speed, or a length not above
/// `ITEM_SPACING`) clear the world, so later calls become no-ops.
#[no_mangle]
pub extern "C" fn axiom_init(lane_count: u32, lane_length: u32, lane_speed: u32) {
    let world = build_world(lane_count, lane_length, lane_speed);
    WORLD.with(|cell| {
        *cell.borrow_mut() = world;
    });
}

/// Advances the world by `ticks` fixed steps.
#[no_mangle]
pub extern "C" fn axiom_step(ticks: u32) {
    with_world(|w| {
        for _ in 0..ticks {
            w.step();
        }
    });
}

/// Offers one item (an `ItemId`) to the head of the chain. Returns 1 if the
/// head lane accepted it, 0 otherwise.
#[no_mangle]
pub extern "C" fn axiom_feed(item: u32) -> u32 {
    let Ok(id) = u16::try_from(item) else {
        return 0;
    };
    with_world(|w| match w.chains_mut().first_mut() {
        Some(chain) => u32::from(chain.try_feed(ItemId(id))),
        None => 0,
    })
    .unwrap_or(0)
}

/// Low 32 bits of the world's state hash.
#[no_mangle]
pub extern "C" fn axiom_state_hash_lo() -> u32 {
    split_hash().0
}

/// High 32 bits of the world's state hash.
#[no_mangle]
pub extern "C" fn axiom_state_hash_hi() -> u32 {
    split_hash().1
}

/// Items that have left the final lane of the chain, saturated to `u32`.
#[no_mangle]
pub extern "C" fn axiom_sunk() -> u32 {
    with_world(|w| w.chains().first().map_or(0, |c| c.sunk())).map_or(0, saturate_u32)
}

/// Fixed steps taken since `axiom_init`, saturated to `u32`.
#[no_mangle]
pub extern "C" fn axiom_tick_count() -> u32 {
    with_world(|w| w.tick_count()).map_or(0, saturate_u32)
}

/// Copies every item's position into the snapshot buffer and returns how many
/// items there are, saturated to `u32`. Call once per frame, then read the
/// items with the `axiom_item_*` getters. Returns 0 if there is no world.
#[no_mangle]
pub extern "C" fn axiom_snapshot_items() -> u32 {
    let items = with_world(|w| collect_items(w.chains())).unwrap_or_default();
    let count = saturate_u32(items.len() as u64);
    SNAPSHOT.with(|cell| {
        if let Ok(mut slot) = cell.try_borrow_mut() {
            *slot = items;
        }
    });
    count
}

/// Reads one field of snapshot item `index`, or 0 if the index is out of range.
fn snapshot_field(index: u32, field: impl FnOnce(&ItemSnapshot) -> u32) -> u32 {
    SNAPSHOT.with(|cell| {
        cell.try_borrow()
            .ok()
            .and_then(|v| v.get(index as usize).map(field))
            .unwrap_or(0)
    })
}

/// Item id of snapshot item `index`.
#[no_mangle]
pub extern "C" fn axiom_item_id(index: u32) -> u32 {
    snapshot_field(index, |s| u32::from(s.item))
}

/// Lane index of snapshot item `index`.
#[no_mangle]
pub extern "C" fn axiom_item_lane(index: u32) -> u32 {
    snapshot_field(index, |s| s.lane)
}

/// Offset along its lane of snapshot item `index`.
#[no_mangle]
pub extern "C" fn axiom_item_offset(index: u32) -> u32 {
    snapshot_field(index, |s| s.offset)
}

/// Hashes the world state and splits the result into `(lo, hi)`. Each
/// exported getter calls this, so the hash is computed once per getter.
fn split_hash() -> (u32, u32) {
    split_u64(with_world(|w| w.state_hash()).unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_u64_gives_low_then_high_halves() {
        assert_eq!(split_u64(0), (0, 0));
        assert_eq!(split_u64(1), (1, 0));
        assert_eq!(split_u64(1 << 32), (0, 1));
        assert_eq!(split_u64(0x1234_5678_9abc_def0), (0x9abc_def0, 0x1234_5678));
        assert_eq!(split_u64(u64::MAX), (u32::MAX, u32::MAX));
    }

    #[test]
    fn split_u64_round_trips() {
        for v in [
            0u64,
            1,
            0xcbf2_9ce4_8422_2325,
            0x0123_4567_89ab_cdef,
            u64::MAX,
        ] {
            let (lo, hi) = split_u64(v);
            assert_eq!((u64::from(hi) << 32) | u64::from(lo), v);
        }
    }

    #[test]
    fn saturate_passes_small_values_and_clamps_large_ones() {
        assert_eq!(saturate_u32(0), 0);
        assert_eq!(saturate_u32(u64::from(u32::MAX)), u32::MAX);
        assert_eq!(saturate_u32(u64::from(u32::MAX) + 1), u32::MAX);
        assert_eq!(saturate_u32(u64::MAX), u32::MAX);
    }

    #[test]
    fn lane_capacity_counts_minimum_spaced_slots() {
        // Offsets 0, 4, ..., up to length - ITEM_SPACING.
        assert_eq!(lane_capacity(ITEM_SPACING + 1), 1);
        assert_eq!(lane_capacity(40), 10);
        assert_eq!(lane_capacity(41), 10);
    }

    #[test]
    fn layout_validation_rejects_degenerate_input() {
        assert!(!is_valid_layout(0, 40, 2), "zero lanes");
        assert!(
            !is_valid_layout(1, ITEM_SPACING, 2),
            "length must exceed spacing"
        );
        assert!(!is_valid_layout(1, 40, 0), "zero speed");
        assert!(
            is_valid_layout(1, ITEM_SPACING + 1, 1),
            "smallest valid lane"
        );
    }

    #[test]
    fn build_world_creates_one_chain_of_requested_lanes() {
        let w = build_world(3, 40, 2).expect("valid layout");
        assert_eq!(w.chains().len(), 1);
        let lanes = w.chains()[0].lanes();
        assert_eq!(lanes.len(), 3);
        for lane in lanes {
            assert_eq!(lane.length(), 40);
            assert!(lane.is_empty());
        }
        assert_eq!(w.tick_count(), 0);
    }

    #[test]
    fn build_world_rejects_invalid_layout() {
        assert!(build_world(0, 40, 2).is_none());
        assert!(build_world(2, ITEM_SPACING, 2).is_none());
        assert!(build_world(2, 40, 0).is_none());
    }

    #[test]
    fn same_layout_gives_same_initial_hash() {
        let a = build_world(2, 40, 2).expect("valid layout");
        let b = build_world(2, 40, 2).expect("valid layout");
        assert_eq!(a.state_hash(), b.state_hash());
        let c = build_world(3, 40, 2).expect("valid layout");
        assert_ne!(a.state_hash(), c.state_hash());
    }

    #[test]
    fn collect_items_is_empty_for_an_empty_world() {
        let w = build_world(2, 40, 2).expect("valid layout");
        assert!(collect_items(w.chains()).is_empty());
    }

    #[test]
    fn collect_items_reports_item_id_lane_and_offset() {
        let mut w = build_world(2, 40, 2).expect("valid layout");
        let head = w.chains_mut().first_mut().expect("one chain");
        assert!(head.try_feed(ItemId(7)));
        assert_eq!(
            collect_items(w.chains()),
            vec![ItemSnapshot {
                item: 7,
                lane: 0,
                offset: 0
            }]
        );
    }

    #[test]
    fn collect_items_follows_the_item_as_it_moves() {
        let mut w = build_world(2, 40, 2).expect("valid layout");
        assert!(w
            .chains_mut()
            .first_mut()
            .expect("chain")
            .try_feed(ItemId(3)));
        w.step();
        let items = collect_items(w.chains());
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].offset, 2, "lane speed is 2 per tick");
    }

    #[test]
    fn collect_items_numbers_lanes_across_chains() {
        let lane = || BeltLane::with_capacity(40, 2, 10);
        let chains = vec![
            BeltChain::new(vec![lane(), lane()]),
            BeltChain::new(vec![lane()]),
        ];
        let mut chains = chains;
        assert!(chains[1].try_feed(ItemId(9)));
        let items = collect_items(&chains);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].lane, 2, "third lane overall");
    }
}
