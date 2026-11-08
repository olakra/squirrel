//! Offline belt throughput benchmarks.
//!
//! A std-only custom harness (`harness = false` in Cargo.toml), so it runs
//! without a crate registry. Criterion is deliberately not used.
//!
//! Usage:
//!     cargo bench --bench belt_throughput             # full sizes
//!     cargo bench --bench belt_throughput -- --quick  # ~10x smaller smoke run
//!
//! Output is one metric per line:
//!     bench <scenario>.<metric> <value> <unit>
//! Each scenario reports `ns_per_tick` (median), `items_per_sec` and
//! `state_hash` (FNV-1a 64 of the post-run state).
//!
//! Only the stable public API is used: lane and chain construction, pushes,
//! pops, advance and tick, sink control, `lanes()` and `lanes_mut()`, and
//! `World::new`, `state_hash`.

use std::hint::black_box;
use std::time::Instant;

use axiom::{BeltChain, BeltLane, ItemId, World};

const SCENARIO_STRESS: &str = "stress_100k_items_10k_segments";
const SCENARIO_BLOCKED: &str = "blocked_backpressure";
const SCENARIO_REAR_PUSH: &str = "rear_push";

/// Geometry for the belt scenarios. At length 60 and spacing 4 a lane packs at
/// most 15 items, so `LANE_CAPACITY` is never outgrown.
const LANE_LENGTH: u32 = 60;
const LANE_SPEED: u32 = 2;
const LANE_CAPACITY: usize = 16;
const LANES_PER_CHAIN: usize = 100;

/// The rear_push lane is long so its queue holds about 100 items. That keeps
/// the rear insertion (a memmove) representative of a busy lane.
const REAR_LANE_LENGTH: u32 = 400;
const REAR_LANE_SPEED: u32 = 4;
const REAR_LANE_CAPACITY: usize = 128;

const WARMUP_RUNS: usize = 1;
const MEASURED_RUNS_FULL: usize = 5;
const MEASURED_RUNS_QUICK: usize = 3;
/// Safety valve so a fill that never reaches its target fails loudly.
const FILL_ROUND_BUDGET: u64 = 200_000;
/// Untimed settle ticks with the sink closed. Items need about 3,000 ticks to
/// cross a 100-lane chain at speed 2, so 6,000 gives a 2x margin and leaves
/// every chain tail backed up and stationary.
const SETTLE_TICKS: u64 = 6_000;

#[derive(Clone, Copy)]
struct Sizes {
    /// Belt lanes (segments) in the layout.
    lanes: usize,
    /// Items the fill phase places on belts before timing starts.
    items: usize,
    /// Ticks in each timed belt run.
    ticks: u64,
    /// Push, advance, pop cycles in the rear_push scenario.
    rear_cycles: u64,
}

const FULL: Sizes = Sizes {
    lanes: 10_000,
    items: 100_000,
    ticks: 1_000,
    rear_cycles: 1_000_000,
};

const QUICK: Sizes = Sizes {
    lanes: 1_000,
    items: 10_000,
    ticks: 100,
    rear_cycles: 100_000,
};

/// Result of one timed run.
#[derive(Clone, Copy)]
struct Run {
    /// Wall time per tick. For rear_push, one tick is one push/advance/pop cycle.
    ns_per_tick: f64,
    /// Item moves per tick, used to derive items/sec.
    items_per_tick: f64,
    /// Fingerprint of the state after the run.
    hash: u64,
}

/// Summary across the measured runs of one scenario.
struct Summary {
    ns_per_tick: f64,
    items_per_tick: f64,
    hash: u64,
}

fn main() {
    let quick = std::env::args().any(|arg| arg == "--quick");
    let sizes = if quick { QUICK } else { FULL };
    let measured = if quick {
        MEASURED_RUNS_QUICK
    } else {
        MEASURED_RUNS_FULL
    };

    // The fill is untimed and happens once. Each timed run works on a clone.
    let filled = build_filled_layout(sizes);

    let stress = summarize(measured, || run_belts(&filled, sizes.ticks, true));
    report(SCENARIO_STRESS, &stress);

    let blocked = summarize(measured, || run_belts(&filled, sizes.ticks, false));
    report(SCENARIO_BLOCKED, &blocked);

    let rear = summarize(measured, || run_rear_push(sizes.rear_cycles));
    report(SCENARIO_REAR_PUSH, &rear);
}

/// Runs warmup, then `measured` timed runs. Panics if the state hash differs
/// between runs, since that would mean the simulation is not deterministic.
fn summarize(measured: usize, mut run: impl FnMut() -> Run) -> Summary {
    for _ in 0..WARMUP_RUNS {
        black_box(run());
    }
    let runs: Vec<Run> = (0..measured).map(|_| run()).collect();
    let hash = runs[0].hash;
    assert!(
        runs.iter().all(|r| r.hash == hash),
        "state hash differs between runs: simulation is not deterministic"
    );
    Summary {
        ns_per_tick: median(runs.iter().map(|r| r.ns_per_tick).collect()),
        items_per_tick: runs[0].items_per_tick,
        hash,
    }
}

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    let mid = values.len() / 2;
    if values.len().is_multiple_of(2) {
        (values[mid - 1] + values[mid]) / 2.0
    } else {
        values[mid]
    }
}

fn report(scenario: &str, summary: &Summary) {
    let items_per_sec = summary.items_per_tick * 1e9 / summary.ns_per_tick;
    println!(
        "bench {scenario}.ns_per_tick {:.1} ns/tick",
        summary.ns_per_tick
    );
    println!("bench {scenario}.items_per_sec {items_per_sec:.0} items/s");
    println!(
        "bench {scenario}.state_hash 0x{:016x} fnv1a64",
        summary.hash
    );
}

// ---------------------------------------------------------------------------
// Layout construction (untimed)
// ---------------------------------------------------------------------------

fn new_chain() -> BeltChain {
    let lanes = (0..LANES_PER_CHAIN)
        .map(|_| BeltLane::with_capacity(LANE_LENGTH, LANE_SPEED, LANE_CAPACITY))
        .collect();
    BeltChain::new(lanes)
}

/// Builds `lanes / LANES_PER_CHAIN` chains and feeds them until `items` items
/// sit on belts. The sink stays closed throughout, so nothing is lost. Feeding
/// then stops and the queues settle until every chain tail is backed up. The
/// result is the compressed, stationary layout that the blocked scenario times.
fn build_filled_layout(sizes: Sizes) -> Vec<BeltChain> {
    let chain_count = sizes.lanes / LANES_PER_CHAIN;
    let mut chains: Vec<BeltChain> = (0..chain_count).map(|_| new_chain()).collect();
    for chain in &mut chains {
        chain.set_sink_closed(true);
    }

    let mut fed: u64 = 0;
    let mut on_belts = 0;
    let mut rounds: u64 = 0;
    while on_belts < sizes.items {
        for chain in &mut chains {
            if chain.try_feed(item_for(fed)) {
                fed += 1;
            }
        }
        for chain in &mut chains {
            chain.tick();
        }
        on_belts = items_on_belts(&chains);
        rounds += 1;
        assert!(
            rounds < FILL_ROUND_BUDGET,
            "fill stalled before reaching the item target"
        );
    }

    for _ in 0..SETTLE_TICKS {
        for chain in &mut chains {
            chain.tick();
        }
    }
    chains
}

/// Cycles through seven item kinds so the hashed state is not uniform.
fn item_for(n: u64) -> ItemId {
    ItemId((n % 7) as u16)
}

fn items_on_belts(chains: &[BeltChain]) -> usize {
    chains
        .iter()
        .flat_map(|chain| chain.lanes())
        .map(BeltLane::len)
        .sum()
}

// ---------------------------------------------------------------------------
// Timed scenarios
// ---------------------------------------------------------------------------

/// Times `ticks` chain ticks on a clone of the filled layout. `open_sink`
/// selects the stress run (exit open, items flow) or the blocked run (exit
/// closed, everything stays compressed).
///
/// Items/sec counts item moves. Each tick touches every item on the belts, so
/// items_per_tick is the average item count at the start and end of the timed
/// window. This is an approximation when items sink during the run.
fn run_belts(filled: &[BeltChain], ticks: u64, open_sink: bool) -> Run {
    let mut chains = filled.to_vec();
    for chain in &mut chains {
        chain.set_sink_closed(!open_sink);
    }

    // A closed sink on a settled layout must not change state. Check that
    // outside the timed window so the blocked scenario validates itself.
    let stationary_before = (!open_sink).then(|| World::new(chains.clone()).state_hash());

    let before = items_on_belts(&chains);
    let elapsed_ns = time_ticks(&mut chains, ticks);
    let after = items_on_belts(&chains);

    let hash = World::new(chains).state_hash();
    if let Some(expected) = stationary_before {
        assert_eq!(hash, expected, "blocked layout moved: it was not settled");
    }

    Run {
        ns_per_tick: elapsed_ns / ticks as f64,
        items_per_tick: (before + after) as f64 / 2.0,
        // World's tick counter stays at 0 because the timed loop drives the
        // chains directly. The hash still covers all lane and sink state.
        hash,
    }
}

/// The timed loop: one `BeltChain::tick` per chain per tick, which is what
/// `World::step` does minus its tick counter.
fn time_ticks(chains: &mut [BeltChain], ticks: u64) -> f64 {
    let start = Instant::now();
    for _ in 0..ticks {
        for chain in chains.iter_mut() {
            chain.tick();
        }
    }
    start.elapsed().as_nanos() as f64
}

/// Hot path: `try_push_rear`, `advance` and `pop_front_if_ready` on one lane.
/// Each cycle is one tick. Pushes that find no room are simply not counted.
fn run_rear_push(cycles: u64) -> Run {
    let lane = BeltLane::with_capacity(REAR_LANE_LENGTH, REAR_LANE_SPEED, REAR_LANE_CAPACITY);
    let mut chain = BeltChain::new(vec![lane]);
    let lane = &mut chain.lanes_mut()[0];

    let mut pushed: u64 = 0;
    let mut popped: u64 = 0;
    let start = Instant::now();
    for i in 0..cycles {
        if lane.try_push_rear(item_for(i)) {
            pushed += 1;
        }
        lane.advance();
        if lane.pop_front_if_ready().is_some() {
            popped += 1;
        }
    }
    let elapsed_ns = start.elapsed().as_nanos() as f64;

    assert_eq!(
        pushed,
        popped + lane.len() as u64,
        "rear_push lost or duplicated items"
    );

    Run {
        ns_per_tick: elapsed_ns / cycles as f64,
        items_per_tick: pushed as f64 / cycles as f64,
        hash: World::new(vec![chain]).state_hash(),
    }
}
