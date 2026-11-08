//! Foundational types shared by every simulation module.
//!
//! The simulation uses discrete integer units only. No floating point is
//! used anywhere in simulation state, so results are bit-identical across
//! runs and platforms.

pub mod hash;
pub mod ids;
