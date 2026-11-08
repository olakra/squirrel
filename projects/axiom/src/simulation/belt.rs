//! Interval-queue belt model.
//!
//! Items are never separate actors. A lane stores an ascending sequence of
//! `(item, offset)` pairs. Offsets are discrete distance units along the lane.
//!
//! Ordering convention (logical order, as seen through `BeltLane::items`):
//! - The first item yielded is the rearmost item (smallest offset).
//! - The last item yielded is the front item (largest offset), the one that
//!   reaches the end of the lane first.
//!
//! Storage is a fixed-capacity ring buffer, so rear insertion and front
//! removal are both O(1) and no memmove is ever needed.
//!
//! Per tick, items advance front to back. Each item moves by `speed`, but
//! never past `next_item.offset - ITEM_SPACING`. When the front item is at
//! the end of the lane and downstream refuses it, the whole queue backs up
//! in one O(N) pass with no allocation.

use crate::core::ids::ItemId;

/// Minimum distance between adjacent items, in distance units.
pub const ITEM_SPACING: u32 = 4;

/// Filler for unused ring slots. Never observed through the public API.
const EMPTY_SLOT: BeltItem = BeltItem {
    item: ItemId(0),
    offset: 0,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BeltItem {
    pub item: ItemId,
    /// Distance from the start of this lane.
    pub offset: u32,
}

/// A lane backed by a fixed-capacity ring buffer.
///
/// Ring invariants:
/// - `buf.len()` is the capacity. It is set once in `with_capacity` and never
///   changes, so the tick path never reallocates.
/// - `len <= buf.len()`. Pushes are refused when the lane is full.
/// - Logical index 0 is the rear item and sits at physical slot `head`.
///   Logical index `i` sits at physical slot `(head + i) mod capacity`.
///   Logical index `len - 1` is the front item.
/// - Physical slots outside the live range hold stale data and are never read.
/// - `head < capacity` whenever `capacity > 0`.
#[derive(Clone, Debug)]
pub struct BeltLane {
    buf: Vec<BeltItem>,
    /// Physical slot of the rear item. Meaningful only when `len > 0`.
    head: usize,
    /// Number of live items.
    len: usize,
    /// Total lane length in distance units.
    length: u32,
    /// Distance units advanced per tick.
    speed: u32,
}

impl BeltLane {
    /// Creates a lane with a fixed capacity, allocated here and nowhere else.
    /// A lane holds at most `capacity` items. Pushing into a full lane returns
    /// `false` and leaves the lane unchanged.
    pub fn with_capacity(length: u32, speed: u32, capacity: usize) -> Self {
        assert!(
            length > ITEM_SPACING,
            "lane must be longer than ITEM_SPACING"
        );
        assert!(speed > 0, "lane speed must be positive");
        Self {
            buf: vec![EMPTY_SLOT; capacity],
            head: 0,
            len: 0,
            length,
            speed,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn length(&self) -> u32 {
        self.length
    }

    /// Iterates the live items in logical order, rear to front.
    #[inline]
    pub fn items(&self) -> impl Iterator<Item = &BeltItem> + '_ {
        (0..self.len).map(move |i| &self.buf[self.slot(i)])
    }

    /// Maps a logical index (0 = rear) to a physical slot in `buf`.
    ///
    /// Requires `index < self.len`. Since `head < cap` and `index < cap`, the
    /// sum is below `2 * cap`, so one conditional subtraction wraps it. This
    /// avoids a division or modulo on every access.
    #[inline]
    fn slot(&self, index: usize) -> usize {
        debug_assert!(index < self.len, "logical index out of range");
        let cap = self.buf.len();
        let p = self.head + index;
        if p >= cap {
            p - cap
        } else {
            p
        }
    }

    /// The position the front item stops at when blocked: the lane end minus
    /// one spacing, so a blocked queue stays visibly separated.
    #[inline]
    fn front_cap(&self) -> u32 {
        self.length - ITEM_SPACING
    }

    /// True if the front item is at the exit and could be handed downstream.
    #[inline]
    pub fn front_ready(&self) -> bool {
        self.len > 0 && self.buf[self.slot(self.len - 1)].offset >= self.front_cap()
    }

    /// Whether an item can enter at the rear (offset 0). False when the lane
    /// is full or when the rear item is still too close to offset 0.
    #[inline]
    pub fn can_accept_rear(&self) -> bool {
        if self.len == self.buf.len() {
            return false;
        }
        self.len == 0 || self.buf[self.head].offset >= ITEM_SPACING
    }

    /// Inserts an item at the rear. Returns `false` if there is no room, or if
    /// the lane is at capacity. O(1), no allocation.
    pub fn try_push_rear(&mut self, item: ItemId) -> bool {
        if !self.can_accept_rear() {
            return false;
        }
        // Rear is the slot just before the current head, wrapping at zero.
        self.head = if self.head == 0 {
            self.buf.len() - 1
        } else {
            self.head - 1
        };
        self.buf[self.head] = BeltItem { item, offset: 0 };
        self.len += 1;
        true
    }

    /// Removes and returns the front item if it is ready to exit. O(1).
    pub fn pop_front_if_ready(&mut self) -> Option<ItemId> {
        if !self.front_ready() {
            return None;
        }
        let it = self.buf[self.slot(self.len - 1)];
        self.len -= 1;
        Some(it.item)
    }

    /// Advances all items one tick, front to back, with spacing compression.
    pub fn advance(&mut self) {
        let n = self.len;
        if n == 0 {
            return;
        }
        let speed = self.speed;
        let front_cap = self.front_cap();

        // Front item: capped at the end of the lane.
        {
            let s = self.slot(n - 1);
            let it = &mut self.buf[s];
            let target = it.offset.saturating_add(speed).min(front_cap);
            // Never move backward, even if the cap is already behind us.
            it.offset = it.offset.max(target);
        }

        // Each following item: capped one spacing behind the item ahead.
        // Iterating from the front uses already-updated offsets, which
        // produces the compression wave.
        for i in (0..n - 1).rev() {
            let ahead = self.buf[self.slot(i + 1)].offset;
            let s = self.slot(i);
            let it = &mut self.buf[s];
            let cap = ahead.saturating_sub(ITEM_SPACING);
            let target = it.offset.saturating_add(speed).min(cap);
            it.offset = it.offset.max(target);
        }
    }

    /// Hashes the lane state into `h` in logical order, rear to front.
    pub fn hash_into(&self, h: &mut crate::core::hash::Fnv1a) {
        h.write_u32(self.length);
        h.write_u32(self.speed);
        h.write_u32(self.len as u32);
        for it in self.items() {
            h.write_u16(it.item.0);
            h.write_u32(it.offset);
        }
    }
}

/// A run of lanes where each lane's exit feeds the next lane's rear.
#[derive(Clone, Debug, Default)]
pub struct BeltChain {
    lanes: Vec<BeltLane>,
    /// Items that left the final lane, kept as a sink count for tests and
    /// throughput metrics.
    sunk: u64,
    /// When true, the final lane's exit is blocked (a full consumer). Items
    /// back up instead of sinking. Defaults to false (sink always open).
    sink_closed: bool,
}

impl BeltChain {
    pub fn new(lanes: Vec<BeltLane>) -> Self {
        Self {
            lanes,
            sunk: 0,
            sink_closed: false,
        }
    }

    /// Opens or closes the final exit. A closed sink models a consumer that
    /// cannot accept items, which produces back-pressure up the chain.
    pub fn set_sink_closed(&mut self, closed: bool) {
        self.sink_closed = closed;
    }

    pub fn lanes(&self) -> &[BeltLane] {
        &self.lanes
    }

    pub fn lanes_mut(&mut self) -> &mut [BeltLane] {
        &mut self.lanes
    }

    pub fn sunk(&self) -> u64 {
        self.sunk
    }

    /// Tries to feed one item into the head of the chain.
    pub fn try_feed(&mut self, item: ItemId) -> bool {
        match self.lanes.first_mut() {
            Some(head) => head.try_push_rear(item),
            None => false,
        }
    }

    /// One tick. Downstream lanes are processed first, so a lane frees space
    /// before its upstream neighbour tries to hand items over.
    pub fn tick(&mut self) {
        let n = self.lanes.len();
        for i in (0..n).rev() {
            self.lanes[i].advance();
            if self.lanes[i].front_ready() {
                if i + 1 < n {
                    // Transfer only if the next lane's rear has room.
                    if self.lanes[i + 1].can_accept_rear() {
                        let item = self.lanes[i]
                            .pop_front_if_ready()
                            .expect("front_ready was true");
                        let ok = self.lanes[i + 1].try_push_rear(item);
                        debug_assert!(ok, "can_accept_rear was true");
                    }
                } else if !self.sink_closed {
                    // Final lane: items sink into the consumer when it is open.
                    if self.lanes[i].pop_front_if_ready().is_some() {
                        self.sunk += 1;
                    }
                }
            }
        }
    }

    pub fn hash_into(&self, h: &mut crate::core::hash::Fnv1a) {
        h.write_u64(self.sunk);
        h.write_u32(self.lanes.len() as u32);
        for lane in &self.lanes {
            lane.hash_into(h);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lane(length: u32, speed: u32) -> BeltLane {
        BeltLane::with_capacity(length, speed, 64)
    }

    /// Deterministic LCG for test sequences. No external crate.
    struct Lcg(u64);

    impl Lcg {
        fn next_u32(&mut self) -> u32 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (self.0 >> 33) as u32
        }
    }

    /// Reference lane: the original Vec-based model, with the same capacity
    /// rule as the ring. Kept deliberately naive so it is easy to trust.
    struct RefLane {
        items: Vec<BeltItem>,
        cap: usize,
        length: u32,
        speed: u32,
    }

    impl RefLane {
        fn new(length: u32, speed: u32, cap: usize) -> Self {
            Self {
                items: Vec::new(),
                cap,
                length,
                speed,
            }
        }

        fn front_ready(&self) -> bool {
            self.items
                .last()
                .is_some_and(|it| it.offset >= self.length - ITEM_SPACING)
        }

        fn can_accept_rear(&self) -> bool {
            self.items.len() < self.cap
                && self
                    .items
                    .first()
                    .is_none_or(|it| it.offset >= ITEM_SPACING)
        }

        fn try_push_rear(&mut self, item: ItemId) -> bool {
            if !self.can_accept_rear() {
                return false;
            }
            self.items.insert(0, BeltItem { item, offset: 0 });
            true
        }

        fn pop_front_if_ready(&mut self) -> Option<ItemId> {
            if self.front_ready() {
                self.items.pop().map(|it| it.item)
            } else {
                None
            }
        }

        fn advance(&mut self) {
            let n = self.items.len();
            if n == 0 {
                return;
            }
            let front_cap = self.length - ITEM_SPACING;
            let speed = self.speed;
            {
                let it = &mut self.items[n - 1];
                let target = it.offset.saturating_add(speed).min(front_cap);
                it.offset = it.offset.max(target);
            }
            for i in (0..n - 1).rev() {
                let ahead = self.items[i + 1].offset;
                let cap = ahead.saturating_sub(ITEM_SPACING);
                let it = &mut self.items[i];
                let target = it.offset.saturating_add(speed).min(cap);
                it.offset = it.offset.max(target);
            }
        }
    }

    #[test]
    fn push_places_item_at_rear_offset_zero() {
        let mut l = lane(100, 1);
        assert!(l.try_push_rear(ItemId(7)));
        let got: Vec<BeltItem> = l.items().copied().collect();
        assert_eq!(
            got,
            vec![BeltItem {
                item: ItemId(7),
                offset: 0
            }]
        );
    }

    #[test]
    fn push_rejected_when_rear_is_too_close() {
        let mut l = lane(100, 1);
        assert!(l.try_push_rear(ItemId(1)));
        // Rear item is still at offset 0, so there is no room yet.
        assert!(!l.try_push_rear(ItemId(2)));
        // After ITEM_SPACING ticks at speed 1, the rear has moved far enough.
        for _ in 0..ITEM_SPACING {
            l.advance();
        }
        assert!(l.try_push_rear(ItemId(2)));
    }

    #[test]
    fn items_keep_spacing_and_stay_ordered() {
        let mut l = lane(200, 2);
        for id in 0..5u16 {
            while !l.try_push_rear(ItemId(id)) {
                l.advance();
            }
        }
        for _ in 0..300 {
            l.advance();
            let offs: Vec<u32> = l.items().map(|i| i.offset).collect();
            for w in offs.windows(2) {
                assert!(w[1] >= w[0] + ITEM_SPACING, "spacing violated: {offs:?}");
            }
        }
    }

    #[test]
    fn blocked_front_compresses_and_never_exceeds_cap() {
        // No downstream: the queue should compress at the cap and stop.
        let mut l = lane(50, 3);
        for id in 0..6u16 {
            while !l.try_push_rear(ItemId(id)) {
                l.advance();
            }
        }
        for _ in 0..500 {
            l.advance();
        }
        let front = l.items().last().unwrap().offset;
        assert_eq!(front, l.length() - ITEM_SPACING, "front must stop at cap");
        for it in l.items() {
            assert!(it.offset <= l.length() - ITEM_SPACING);
        }
        assert_eq!(l.len(), 6, "blocked lane must not lose items");
    }

    #[test]
    fn chain_moves_items_downstream_and_sinks_them() {
        let mut chain = BeltChain::new(vec![lane(40, 2), lane(40, 2), lane(40, 2)]);
        let mut fed = 0u64;
        for t in 0..400u32 {
            if t % 6 == 0 && chain.try_feed(ItemId(1)) {
                fed += 1;
            }
            chain.tick();
        }
        let in_flight: usize = chain.lanes().iter().map(|l| l.len()).sum();
        assert_eq!(fed, chain.sunk() + in_flight as u64, "items conserved");
        assert!(chain.sunk() > 0, "items should reach the sink");
    }

    #[test]
    fn backpressure_holds_items_without_loss_and_resumes_when_sink_opens() {
        let mut chain = BeltChain::new(vec![lane(40, 2), lane(40, 2)]);
        chain.set_sink_closed(true);

        // Feed until the head refuses, then keep ticking so the queue fills.
        let mut fed = 0u64;
        for _ in 0..400 {
            if chain.try_feed(ItemId(2)) {
                fed += 1;
            }
            chain.tick();
        }
        let held: usize = chain.lanes().iter().map(|l| l.len()).sum();
        assert_eq!(chain.sunk(), 0, "closed sink must not consume");
        assert_eq!(fed, held as u64, "every fed item is still held");

        // Blocked state is stable: nothing moves or disappears.
        let snapshot: Vec<Vec<BeltItem>> = chain
            .lanes()
            .iter()
            .map(|l| l.items().copied().collect())
            .collect();
        for _ in 0..100 {
            chain.tick();
        }
        let after: Vec<Vec<BeltItem>> = chain
            .lanes()
            .iter()
            .map(|l| l.items().copied().collect())
            .collect();
        assert_eq!(snapshot, after, "blocked chain must be stationary");

        // Reopen the sink: the queue drains and totals are conserved.
        chain.set_sink_closed(false);
        for _ in 0..400 {
            chain.tick();
        }
        let left: usize = chain.lanes().iter().map(|l| l.len()).sum();
        assert_eq!(
            fed,
            chain.sunk() + left as u64,
            "items conserved after resume"
        );
        assert!(chain.sunk() > 0, "items drain once the sink reopens");
    }

    #[test]
    fn lane_hash_changes_when_state_changes() {
        use crate::core::hash::Fnv1a;
        let mut a = lane(100, 1);
        let mut b = lane(100, 1);
        a.try_push_rear(ItemId(3));
        let mut ha = Fnv1a::new();
        let mut hb = Fnv1a::new();
        a.hash_into(&mut ha);
        b.hash_into(&mut hb);
        assert_ne!(ha.finish(), hb.finish());
        b.try_push_rear(ItemId(3));
        let mut hb2 = Fnv1a::new();
        b.hash_into(&mut hb2);
        assert_eq!(ha.finish(), hb2.finish());
    }

    #[test]
    fn differential_ring_matches_vec_reference_over_10k_steps() {
        // Capacity 6 on a 60-unit lane (physical max would be 15) so the
        // full-lane rejection path is exercised as well as the spacing path.
        const LENGTH: u32 = 60;
        const SPEED: u32 = 3;
        const CAP: usize = 6;
        let mut ring = BeltLane::with_capacity(LENGTH, SPEED, CAP);
        let mut reference = RefLane::new(LENGTH, SPEED, CAP);
        let mut rng = Lcg(0x5EED_CAFE);

        for step in 0..10_000u32 {
            match rng.next_u32() % 8 {
                0..=2 => {
                    let id = ItemId((rng.next_u32() % 1000) as u16);
                    assert_eq!(
                        ring.try_push_rear(id),
                        reference.try_push_rear(id),
                        "push result diverged at step {step}"
                    );
                }
                3..=4 => {
                    assert_eq!(
                        ring.pop_front_if_ready(),
                        reference.pop_front_if_ready(),
                        "pop result diverged at step {step}"
                    );
                }
                _ => {
                    ring.advance();
                    reference.advance();
                }
            }

            let got: Vec<BeltItem> = ring.items().copied().collect();
            assert_eq!(got, reference.items, "item list diverged at step {step}");
            assert_eq!(ring.len(), reference.items.len(), "len at step {step}");
            assert_eq!(
                ring.front_ready(),
                reference.front_ready(),
                "front_ready at step {step}"
            );
            assert_eq!(
                ring.can_accept_rear(),
                reference.can_accept_rear(),
                "can_accept_rear at step {step}"
            );
        }
    }

    #[test]
    fn wraparound_keeps_fifo_order_across_many_wraps() {
        // Three physical slots, so 300 items wrap the ring about 100 times.
        let mut l = BeltLane::with_capacity(20, 4, 3);
        let mut expected: std::collections::VecDeque<u16> = Default::default();
        let mut next_id = 0u16;
        let mut popped = 0u32;

        for _ in 0..20_000 {
            if next_id < 300 && l.try_push_rear(ItemId(next_id)) {
                expected.push_back(next_id);
                next_id += 1;
            }
            l.advance();
            if let Some(id) = l.pop_front_if_ready() {
                assert_eq!(Some(id.0), expected.pop_front(), "FIFO order broken");
                popped += 1;
            }
            // Rear to front is newest to oldest.
            let got: Vec<u16> = l.items().map(|i| i.item.0).collect();
            let want: Vec<u16> = expected.iter().rev().copied().collect();
            assert_eq!(got, want, "ring contents wrong after wrap");
        }
        assert_eq!(popped, 300, "every item must travel through the ring");
    }

    #[test]
    fn push_beyond_capacity_returns_false_without_panicking() {
        let mut l = BeltLane::with_capacity(100, 1, 3);
        for id in 0..3u16 {
            while !l.try_push_rear(ItemId(id)) {
                l.advance();
            }
        }
        assert_eq!(l.len(), 3);
        // Give the rear plenty of room so only capacity can refuse the push.
        for _ in 0..50 {
            l.advance();
        }
        assert!(!l.can_accept_rear(), "full lane must refuse rear entry");
        for _ in 0..10 {
            assert!(!l.try_push_rear(ItemId(9)));
        }
        assert_eq!(l.len(), 3, "refused pushes must not change the lane");

        let mut zero = BeltLane::with_capacity(100, 1, 0);
        assert!(!zero.try_push_rear(ItemId(1)));
        assert!(zero.is_empty());
    }
}
