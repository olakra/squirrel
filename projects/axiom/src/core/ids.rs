//! Compact, copyable identifiers.
//!
//! Newtypes stop an item id from being passed where a lane index is expected.

/// Identifies a kind of item (ore, plate, gear, ...). Stored as `u16` in belt
/// state to keep per-item records small.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ItemId(pub u16);

/// Index of a lane inside a `BeltChain`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct LaneIndex(pub u32);
