//! Project Axiom: deterministic 2D factory simulation core.
//!
//! Module layout (see docs/ARCHITECTURE.md):
//! - `core`: ids, fixed-size integer units, state hashing.
//! - `simulation`: belt lanes, belt chains, and the tick driver.
//!
//! Rendering, threading, and the WASM bindings are later milestones.

pub mod core;
pub mod simulation;

pub use crate::core::hash::Fnv1a;
pub use crate::core::ids::ItemId;
pub use crate::simulation::belt::{BeltChain, BeltLane};
pub use crate::simulation::world::World;

/// Simulation rate. The sim always advances in fixed steps of this rate,
/// independent of render frame rate.
pub const TICKS_PER_SECOND: u32 = 60;
