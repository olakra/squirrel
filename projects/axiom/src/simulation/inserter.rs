//! Discrete inserter state machine.
//!
//! An inserter moves one item per cycle from a source inventory to a target
//! inventory. Phases run in order:
//!
//! `SearchingSource -> PickingUp -> RotatingToTarget -> Dropping -> Resetting`
//!
//! Timing:
//! - `SearchingSource` takes one tick to claim an item when one is free.
//! - Each timed phase lasts exactly its configured tick count.
//! - `Dropping` waits in place while the target is full and holds the item.
//!
//! Reservation: claiming an item increments `Inventory::reserved`. The pickup
//! later consumes that reserved item. Free items are `count - reserved`, so a
//! second inserter on the same source cannot claim the same item.

use crate::core::hash::Fnv1a;

/// A counted stock of identical items with a fixed capacity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Inventory {
    count: u32,
    /// Items claimed by an inserter but not yet picked up. Always `<= count`.
    reserved: u32,
    capacity: u32,
}

impl Inventory {
    /// Creates an inventory holding `count` items, up to `capacity`.
    pub fn new(count: u32, capacity: u32) -> Self {
        assert!(
            count <= capacity,
            "inventory count must not exceed capacity"
        );
        Self {
            count,
            reserved: 0,
            capacity,
        }
    }

    /// Items currently stored, including reserved ones.
    pub fn count(&self) -> u32 {
        self.count
    }

    /// Items claimed for a pickup that has not happened yet.
    pub fn reserved(&self) -> u32 {
        self.reserved
    }

    /// Items that are neither reserved nor claimed.
    pub fn available(&self) -> u32 {
        self.count - self.reserved
    }

    /// Whether one more item fits.
    pub fn has_room(&self) -> bool {
        self.count < self.capacity
    }

    /// Claims one free item for a later pickup. Returns `false` if none is free.
    pub fn reserve_one(&mut self) -> bool {
        if self.available() == 0 {
            return false;
        }
        self.reserved += 1;
        true
    }

    /// Completes the pickup of one previously reserved item.
    fn take_reserved(&mut self) {
        assert!(self.reserved > 0, "pickup requires a reserved item");
        self.reserved -= 1;
        self.count -= 1;
    }

    /// Adds one item if there is room. Returns `false` if the inventory is full.
    fn add_one(&mut self) -> bool {
        if !self.has_room() {
            return false;
        }
        self.count += 1;
        true
    }

    /// Removes one free item, for a consumer. Returns `false` if none is free.
    pub fn remove_one(&mut self) -> bool {
        if self.available() == 0 {
            return false;
        }
        self.count -= 1;
        true
    }

    /// Hashes the inventory state into `h` in a fixed order.
    pub fn hash_into(&self, h: &mut Fnv1a) {
        h.write_u32(self.count);
        h.write_u32(self.reserved);
        h.write_u32(self.capacity);
    }
}

/// The state an inserter is in. Discriminants are stable for hashing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Phase {
    SearchingSource = 0,
    PickingUp = 1,
    RotatingToTarget = 2,
    Dropping = 3,
    Resetting = 4,
}

/// Moves one item per cycle, with fixed tick counts per phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Inserter {
    phase: Phase,
    /// Ticks left in the current timed phase.
    ticks_left: u32,
    pick_ticks: u32,
    rotate_ticks: u32,
    drop_ticks: u32,
    reset_ticks: u32,
}

impl Inserter {
    /// Creates an idle inserter. Every phase must last at least one tick.
    pub fn new(pick_ticks: u32, rotate_ticks: u32, drop_ticks: u32, reset_ticks: u32) -> Self {
        assert!(pick_ticks >= 1, "pick_ticks must be at least 1");
        assert!(rotate_ticks >= 1, "rotate_ticks must be at least 1");
        assert!(drop_ticks >= 1, "drop_ticks must be at least 1");
        assert!(reset_ticks >= 1, "reset_ticks must be at least 1");
        Self {
            phase: Phase::SearchingSource,
            ticks_left: 0,
            pick_ticks,
            rotate_ticks,
            drop_ticks,
            reset_ticks,
        }
    }

    /// The phase the inserter is in.
    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// True while the inserter carries an item (rotating or dropping).
    pub fn is_holding(&self) -> bool {
        matches!(self.phase, Phase::RotatingToTarget | Phase::Dropping)
    }

    /// Advances the inserter by one tick.
    pub fn tick(&mut self, source: &mut Inventory, target: &mut Inventory) {
        match self.phase {
            Phase::SearchingSource => self.search(source),
            Phase::PickingUp => self.pick_up(source),
            Phase::RotatingToTarget => self.rotate(),
            Phase::Dropping => self.drop_into(target),
            Phase::Resetting => self.reset(),
        }
    }

    /// Claims an item if one is free, then starts the pickup.
    fn search(&mut self, source: &mut Inventory) {
        if source.reserve_one() {
            self.enter(Phase::PickingUp, self.pick_ticks);
        }
    }

    /// Finishes the pickup once its ticks are spent.
    fn pick_up(&mut self, source: &mut Inventory) {
        if self.count_down() {
            source.take_reserved();
            self.enter(Phase::RotatingToTarget, self.rotate_ticks);
        }
    }

    /// Swings the held item toward the target.
    fn rotate(&mut self) {
        if self.count_down() {
            self.enter(Phase::Dropping, self.drop_ticks);
        }
    }

    /// Drops the item once the timer has run out and the target has room.
    fn drop_into(&mut self, target: &mut Inventory) {
        if self.count_down() && target.add_one() {
            self.enter(Phase::Resetting, self.reset_ticks);
        }
    }

    /// Returns to searching once the reset ticks are spent.
    fn reset(&mut self) {
        if self.count_down() {
            self.enter(Phase::SearchingSource, 0);
        }
    }

    /// Switches to `phase` with a fresh timer.
    fn enter(&mut self, phase: Phase, ticks: u32) {
        self.phase = phase;
        self.ticks_left = ticks;
    }

    /// Counts one tick off the timer. Returns `true` once it is at zero.
    fn count_down(&mut self) -> bool {
        self.ticks_left = self.ticks_left.saturating_sub(1);
        self.ticks_left == 0
    }

    /// Hashes the inserter state into `h` in a fixed order.
    pub fn hash_into(&self, h: &mut Fnv1a) {
        h.write_u8(self.phase as u8);
        h.write_u32(self.ticks_left);
        h.write_u32(self.pick_ticks);
        h.write_u32(self.rotate_ticks);
        h.write_u32(self.drop_ticks);
        h.write_u32(self.reset_ticks);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pick 2, rotate 3, drop 1, reset 2. One claim tick plus the phases.
    const PICK: u32 = 2;
    const ROTATE: u32 = 3;
    const DROP: u32 = 1;
    const RESET: u32 = 2;
    const CYCLE: u32 = 1 + PICK + ROTATE + DROP + RESET;

    fn inserter() -> Inserter {
        Inserter::new(PICK, ROTATE, DROP, RESET)
    }

    fn run(ins: &mut Inserter, src: &mut Inventory, tgt: &mut Inventory, ticks: u32) {
        for _ in 0..ticks {
            ins.tick(src, tgt);
        }
    }

    #[test]
    fn full_cycle_moves_exactly_one_item() {
        let mut ins = inserter();
        let mut src = Inventory::new(3, 3);
        let mut tgt = Inventory::new(0, 5);

        // The drop lands on the tick before reset, so the last tick only resets.
        run(&mut ins, &mut src, &mut tgt, CYCLE - RESET - 1);
        assert_eq!(tgt.count(), 0, "item is not dropped yet");
        assert_eq!(ins.phase(), Phase::Dropping);

        run(&mut ins, &mut src, &mut tgt, 1);
        assert_eq!(tgt.count(), 1, "drop lands on the drop tick");
        assert_eq!(ins.phase(), Phase::Resetting);

        run(&mut ins, &mut src, &mut tgt, RESET);
        assert_eq!(ins.phase(), Phase::SearchingSource);
        assert_eq!((src.count(), src.reserved()), (2, 0));
        assert_eq!(tgt.count(), 1);

        run(&mut ins, &mut src, &mut tgt, CYCLE);
        assert_eq!(
            (src.count(), tgt.count()),
            (1, 2),
            "one more item per cycle"
        );
    }

    #[test]
    fn no_pick_when_source_is_empty() {
        let mut ins = inserter();
        let mut src = Inventory::new(0, 4);
        let mut tgt = Inventory::new(0, 4);

        run(&mut ins, &mut src, &mut tgt, 50);

        assert_eq!(ins.phase(), Phase::SearchingSource);
        assert!(!ins.is_holding());
        assert_eq!((src.count(), src.reserved()), (0, 0));
        assert_eq!(tgt.count(), 0);
    }

    #[test]
    fn no_drop_when_target_is_full_and_inserter_holds_item() {
        let mut ins = inserter();
        let mut src = Inventory::new(2, 2);
        let mut tgt = Inventory::new(1, 1);

        run(&mut ins, &mut src, &mut tgt, 50);

        assert_eq!(ins.phase(), Phase::Dropping);
        assert!(ins.is_holding(), "item stays in the inserter");
        assert_eq!(tgt.count(), 1, "full target gains nothing");
        assert_eq!(src.count(), 1, "no second pick while holding");

        assert!(tgt.remove_one(), "consumer frees one slot");
        run(&mut ins, &mut src, &mut tgt, 1);
        assert_eq!(tgt.count(), 1, "held item lands in the freed slot");
        assert_eq!(ins.phase(), Phase::Resetting);
        assert_eq!(src.count(), 1);
    }

    #[test]
    fn reservation_prevents_double_take() {
        let mut a = inserter();
        let mut b = inserter();
        let mut src = Inventory::new(1, 1);
        let mut tgt_a = Inventory::new(0, 1);
        let mut tgt_b = Inventory::new(0, 1);

        a.tick(&mut src, &mut tgt_a);
        assert_eq!(a.phase(), Phase::PickingUp);
        assert_eq!((src.count(), src.reserved(), src.available()), (1, 1, 0));

        b.tick(&mut src, &mut tgt_b);
        assert_eq!(
            b.phase(),
            Phase::SearchingSource,
            "reserved item is not free"
        );

        for _ in 0..100 {
            a.tick(&mut src, &mut tgt_a);
            b.tick(&mut src, &mut tgt_b);
        }
        assert_eq!(tgt_a.count() + tgt_b.count(), 1, "exactly one item moved");
        assert_eq!((src.count(), src.reserved()), (0, 0));
    }

    #[test]
    #[should_panic(expected = "pick_ticks must be at least 1")]
    fn zero_phase_length_is_rejected() {
        let _ = Inserter::new(0, 1, 1, 1);
    }

    #[test]
    fn state_hash_tracks_phase() {
        let mut ins = inserter();
        let mut src = Inventory::new(1, 1);
        let mut tgt = Inventory::new(0, 1);
        let mut before = Fnv1a::new();
        ins.hash_into(&mut before);

        ins.tick(&mut src, &mut tgt);
        let mut after = Fnv1a::new();
        ins.hash_into(&mut after);
        assert_ne!(before.finish(), after.finish());
    }
}
