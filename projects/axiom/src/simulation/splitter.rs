//! One-input, two-output splitter for belt lanes.
//!
//! The splitter owns no lanes. Each call moves at most one item from the front
//! of an upstream lane to one of two downstream lanes, so the caller keeps
//! ownership of the belt graph and its tick order.
//!
//! Routing rule:
//! - Each item prefers one output. After an item moves to a side, the other
//!   side becomes the preference. With both outputs open, items strictly
//!   alternate.
//! - If the preferred output refuses (its rear is too close), the item goes to
//!   the other output. The preference still flips.
//! - If both outputs refuse, the item stays on the input lane. Nothing is
//!   dropped.

use crate::core::hash::Fnv1a;
use crate::simulation::belt::BeltLane;

/// One of the two outputs of a splitter.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Side {
    #[default]
    Left = 0,
    Right = 1,
}

impl Side {
    /// The output on the opposite side.
    #[inline]
    pub const fn other(self) -> Side {
        match self {
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }
}

/// Splits one belt into two, alternating deterministically.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Splitter {
    /// Output that receives the next item if it can accept it.
    preferred: Side,
    sent_left: u64,
    sent_right: u64,
}

impl Splitter {
    /// Creates a splitter whose first preference is the left output.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of items delivered to `side` since construction.
    pub fn sent_to(&self, side: Side) -> u64 {
        match side {
            Side::Left => self.sent_left,
            Side::Right => self.sent_right,
        }
    }

    /// Moves the front item of `input` to an open output.
    ///
    /// Returns the side used, or `None` if the input front is not ready or both
    /// outputs refuse. In the `None` case the item stays on `input`.
    pub fn route_front(
        &mut self,
        input: &mut BeltLane,
        left: &mut BeltLane,
        right: &mut BeltLane,
    ) -> Option<Side> {
        if !input.front_ready() {
            return None;
        }
        let side = self.choose_open_side(left, right)?;
        let item = input.pop_front_if_ready()?;
        let target = match side {
            Side::Left => left,
            Side::Right => right,
        };
        let placed = target.try_push_rear(item);
        debug_assert!(placed, "choose_open_side checked can_accept_rear");
        self.record_send(side);
        Some(side)
    }

    /// The preferred output if it accepts, otherwise the other one, otherwise `None`.
    fn choose_open_side(&self, left: &BeltLane, right: &BeltLane) -> Option<Side> {
        if accepts(self.preferred, left, right) {
            Some(self.preferred)
        } else if accepts(self.preferred.other(), left, right) {
            Some(self.preferred.other())
        } else {
            None
        }
    }

    /// Counts an item sent to `side` and flips the preference.
    fn record_send(&mut self, side: Side) {
        match side {
            Side::Left => self.sent_left += 1,
            Side::Right => self.sent_right += 1,
        }
        self.preferred = side.other();
    }

    /// Hashes the splitter state into `h` in a fixed order.
    pub fn hash_into(&self, h: &mut Fnv1a) {
        h.write_u8(self.preferred as u8);
        h.write_u64(self.sent_left);
        h.write_u64(self.sent_right);
    }
}

/// Whether the output on `side` has room at its rear.
fn accepts(side: Side, left: &BeltLane, right: &BeltLane) -> bool {
    match side {
        Side::Left => left.can_accept_rear(),
        Side::Right => right.can_accept_rear(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ids::ItemId;

    const ITEMS: u16 = 101;
    const TICKS: u32 = 1_000;

    fn lane() -> BeltLane {
        BeltLane::with_capacity(20, 2, 16)
    }

    /// Pushes the next item into `input` if its rear has room.
    fn feed_next(input: &mut BeltLane, next_id: &mut u16, total: u16) {
        if *next_id < total && input.try_push_rear(ItemId(*next_id)) {
            *next_id += 1;
        }
    }

    /// Advances a lane and removes every item that reaches its exit.
    fn advance_and_sink(lane: &mut BeltLane) {
        lane.advance();
        while lane.pop_front_if_ready().is_some() {}
    }

    /// Feeds `input` with `ITEMS` items and routes for `TICKS` ticks. The left
    /// output is advanced only when `left_open` is true, so a closed left
    /// output keeps its blocker at the rear.
    fn run(
        s: &mut Splitter,
        input: &mut BeltLane,
        left: &mut BeltLane,
        right: &mut BeltLane,
        open: (bool, bool),
    ) -> Vec<Option<Side>> {
        let (left_open, right_open) = open;
        let mut next_id = 0u16;
        let mut routes = Vec::new();
        for _ in 0..TICKS {
            feed_next(input, &mut next_id, ITEMS);
            input.advance();
            routes.push(s.route_front(input, left, right));
            if left_open {
                advance_and_sink(left);
            }
            if right_open {
                advance_and_sink(right);
            }
        }
        routes
    }

    #[test]
    fn balanced_split_over_n_items_differs_by_at_most_one() {
        let mut s = Splitter::new();
        let (mut input, mut left, mut right) = (lane(), lane(), lane());
        run(&mut s, &mut input, &mut left, &mut right, (true, true));

        assert!(input.is_empty(), "every item must leave the input");
        let l = s.sent_to(Side::Left);
        let r = s.sent_to(Side::Right);
        assert_eq!(l + r, u64::from(ITEMS), "every item is routed exactly once");
        assert!(
            l.abs_diff(r) <= 1,
            "left={l} right={r} must differ by at most 1"
        );
    }

    #[test]
    fn open_outputs_alternate_strictly() {
        let mut s = Splitter::new();
        let (mut input, mut left, mut right) = (lane(), lane(), lane());
        let sides: Vec<Side> = run(&mut s, &mut input, &mut left, &mut right, (true, true))
            .into_iter()
            .flatten()
            .collect();

        assert_eq!(sides.len(), usize::from(ITEMS));
        assert_eq!(sides[0], Side::Left, "first item prefers left");
        for pair in sides.windows(2) {
            assert_ne!(pair[0], pair[1], "sides must alternate: {sides:?}");
        }
    }

    #[test]
    fn blocked_output_routes_everything_to_the_other() {
        let mut s = Splitter::new();
        let (mut input, mut left, mut right) = (lane(), lane(), lane());
        // A blocker at the left rear. The left lane is never advanced, so the
        // blocker never clears and the left output refuses all items.
        assert!(left.try_push_rear(ItemId(900)));

        run(&mut s, &mut input, &mut left, &mut right, (false, true));

        assert_eq!(s.sent_to(Side::Left), 0, "blocked left receives nothing");
        assert_eq!(
            s.sent_to(Side::Right),
            u64::from(ITEMS),
            "right takes all items"
        );
        assert!(input.is_empty());
        assert_eq!(left.len(), 1, "only the blocker stays on the left lane");
    }

    #[test]
    fn both_blocked_holds_the_item_upstream() {
        let mut s = Splitter::new();
        let mut input = lane();
        let (mut left, mut right) = (lane(), lane());
        assert!(input.try_push_rear(ItemId(5)));
        for _ in 0..8 {
            input.advance();
        }
        assert!(
            input.front_ready(),
            "item must be at the exit before routing"
        );
        assert!(left.try_push_rear(ItemId(900)));
        assert!(right.try_push_rear(ItemId(901)));

        for _ in 0..50 {
            assert_eq!(s.route_front(&mut input, &mut left, &mut right), None);
        }
        assert_eq!(input.len(), 1, "item stays upstream while both are blocked");
        assert_eq!(left.len(), 1);
        assert_eq!(right.len(), 1);
        assert_eq!(s.sent_to(Side::Left) + s.sent_to(Side::Right), 0);

        // Clear both blockers. The item then goes to the preferred left output.
        left.advance();
        left.advance();
        right.advance();
        right.advance();
        assert_eq!(
            s.route_front(&mut input, &mut left, &mut right),
            Some(Side::Left)
        );
        assert!(input.is_empty());
    }

    /// Runs a fixed scenario and returns a hash of all state it touched.
    fn scenario_hash(block_right: bool) -> u64 {
        let mut s = Splitter::new();
        let (mut input, mut left, mut right) = (lane(), lane(), lane());
        if block_right {
            assert!(right.try_push_rear(ItemId(900)));
        }
        run(
            &mut s,
            &mut input,
            &mut left,
            &mut right,
            (true, !block_right),
        );

        let mut h = Fnv1a::new();
        s.hash_into(&mut h);
        input.hash_into(&mut h);
        left.hash_into(&mut h);
        right.hash_into(&mut h);
        h.finish()
    }

    #[test]
    fn identical_runs_produce_identical_state_hashes() {
        assert_eq!(scenario_hash(false), scenario_hash(false));
        assert_eq!(scenario_hash(true), scenario_hash(true));
    }

    #[test]
    fn different_routing_produces_different_state_hashes() {
        assert_ne!(scenario_hash(false), scenario_hash(true));
    }
}
