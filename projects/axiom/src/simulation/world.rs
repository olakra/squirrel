//! The world owns all simulation state and advances it in fixed ticks.

use crate::core::hash::Fnv1a;
use crate::simulation::belt::BeltChain;

#[derive(Clone, Debug, Default)]
pub struct World {
    tick: u64,
    chains: Vec<BeltChain>,
}

impl World {
    pub fn new(chains: Vec<BeltChain>) -> Self {
        Self { tick: 0, chains }
    }

    #[inline]
    pub fn tick_count(&self) -> u64 {
        self.tick
    }

    pub fn chains(&self) -> &[BeltChain] {
        &self.chains
    }

    pub fn chains_mut(&mut self) -> &mut [BeltChain] {
        &mut self.chains
    }

    /// Advances one fixed simulation step. Chains are independent in this
    /// milestone, so their order does not affect results. Cross-chain
    /// interactions (inserters, splitters) will need a fixed iteration order.
    pub fn step(&mut self) {
        for c in &mut self.chains {
            c.tick();
        }
        self.tick += 1;
    }

    /// Fingerprint of the full state, including tick count.
    pub fn state_hash(&self) -> u64 {
        let mut h = Fnv1a::new();
        h.write_u64(self.tick);
        h.write_u32(self.chains.len() as u32);
        for c in &self.chains {
            c.hash_into(&mut h);
        }
        h.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ids::ItemId;
    use crate::simulation::belt::BeltLane;

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
    fn determinism_1000_ticks_same_hash_across_runs() {
        let a = scenario();
        let b = scenario();
        assert_eq!(a.tick_count(), 1_000);
        assert_eq!(a.state_hash(), b.state_hash());
    }

    #[test]
    fn hash_detects_a_single_changed_item() {
        let a = scenario();
        let mut b = scenario();
        // Shift one item's offset by one unit to check the hash is sensitive.
        // Items are private, so change the state through a fresh run that
        // feeds one extra item at a different tick.
        let _ = b.chains_mut()[0].try_feed(ItemId(9));
        assert_ne!(a.state_hash(), b.state_hash());
    }
}
