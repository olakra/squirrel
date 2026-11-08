# Performance budget

- **Status:** Baseline from a sandbox quick run. Target hardware not measured yet.
- **Date:** 2026-11-08
- **Tracks:** issue #3. Benchmark gate decision: [ADR 0004](adr/0004-benchmark-regression-gate.md).

## Targets

| Target | Value | Source |
|---|---|---|
| Simulation rate | 60 UPS (updates per second), fixed 60 Hz step | rust-wasm spec, sections 1.1 and 1.2 |
| Render rate | 60 FPS on integrated graphics at 1080p | studio spec, section V |
| Tick budget | 16,666,666 ns per tick (1 s / 60, truncated) | Derived from 60 Hz |
| Active items at 60 UPS | 200,000 items on belts | studio spec, section V |
| Stress scenario | 100,000 items on 10,000 belt segments | rust-wasm spec, benchmarks item |

The rust-wasm and studio specs name 60 Hz but do not state a per-tick millisecond budget. The
16.6 ms figure in the studio spec is input latency (click to visual feedback), not a tick budget.
The tick budget here is derived from 60 Hz.

## Headroom

The team wants the stress tick to stay under half the budget on target hardware, which is
8,333,333 ns. [ADR 0002](adr/0002-tech-stack.md) uses this point as the trigger for the threading
upgrade: it says to adopt SharedArrayBuffer and worker pools only if a measured stress tick on the
Deck passes half the budget. The owner confirms this target.

## Stress scenario

The benchmark is `projects/axiom/benches/belt_throughput.rs`. Its meaning is defined in
[benches/README.md](../../projects/axiom/benches/README.md).

- **Full size:** 10,000 belt lanes in 100 chains of 100 lanes. Each lane is 60 units long at
  speed 2. 100,000 items. 1,000 timed ticks. The exit is open, so items move and drain at the tail.
- **Setup:** untimed. The layout is filled with the exit closed, then settled for 6,000 ticks so
  every chain tail is backed up. Each timed run works on a clone of the settled layout.
- **Metric:** `ns_per_tick` is the median of 5 measured runs after 1 warmup in full mode. Quick mode
  uses 3 measured runs.
- **Quick size:** 1,000 lanes, 10,000 items, 100 ticks. It is a smoke run and is not comparable with
  full size.
- **Related scenarios:** `blocked_backpressure` (same layout, exit closed) and `rear_push` (one lane,
  1,000,000 cycles).

Gap: the benchmark uses 100,000 items. The studio spec target is 200,000 active items at 60 UPS. That
target is not benchmarked. See [v1-scope.md](v1-scope.md), open question 1.

## Reference machine and sandbox baseline

| Item | Value |
|---|---|
| CPU | Intel(R) Xeon(R) Processor @ 2.10GHz, x86_64 (`/proc/cpuinfo`) |
| Logical CPUs | 2 (`nproc`) |
| Memory | 8 GB |
| OS | Ubuntu 24.04.5 LTS, Linux 6.18.44, running as a virtual machine |
| rustc | 1.97.0 (2d8144b78 2026-07-07) |
| cargo | 1.97.0 (c980f4866 2026-06-30) |
| Build profile | `bench`, inherits `release`: opt-level 3, LTO fat, codegen-units 1, panic abort, strip, no debug info |
| Command | `cargo bench --bench belt_throughput -- --quick` |
| Run date | 2026-11-08 |

The crate has no `rust-toolchain.toml`, so rustc is whichever version is installed.

**This sandbox is NOT the target hardware.** It is a 2-vCPU virtual machine. The target is low-end
hardware with integrated graphics (the spec names Intel Iris Xe and AMD Vega class, and standard
consumer laptops) and the Steam Deck. A result here is not a pass on target hardware.

### Quick run (one invocation)

Quick size: 1,000 lanes, 10,000 items, 100 ticks. The values are the medians the harness printed.

| Scenario (quick size) | `ns_per_tick` | `items_per_sec` | `state_hash` (fnv1a64) |
|---|---|---|---|
| `stress_100k_items_10k_segments` | 30006.0 | 324934905 | 0xa65ef336bd29a86b |
| `blocked_backpressure` | 28584.0 | 349846435 | 0x4d4cb4ee8376c07f |
| `rear_push` (100,000 cycles) | 358.2 | 2791495 | 0xd948c3fd2044bcbb |

The harness prints only the median. The spread across the three measured runs is not recorded.

## Result against the budget

- **Quick stress tick:** 30,006 ns, which is 0.18% of the 16,666,666 ns budget. The remaining
  headroom is 16,636,660 ns. The budget is about 555 times the measured tick.
- **This is not the stress scenario.** The quick size has one tenth of the lanes and items of full
  size. Use it only as a smoke signal.
- **Full size on this sandbox: not measured in this session.** [ADR 0002](adr/0002-tech-stack.md)
  records 238,229 ns per tick (1.4% of budget) for full size from an earlier run on this sandbox
  type. That number is not reproduced here, and this document does not rely on it.
- **Status:** sandbox quick size is below budget, so PASS for this machine only. Target hardware
  at full size is NOT RUN. The budget status stays open until the owner runs it.

## Target hardware results

None yet. The owner records them here, one row per run.

## What the owner must do

1. Choose the target machine: a low-end integrated-graphics laptop from the spec, or a Steam Deck.
2. On that machine, run `cargo bench --bench belt_throughput` (full size) five times. Record each
   stress `ns_per_tick`, then the median and the spread (maximum minus minimum).
3. Name the machine (CPU, OS, rustc version) in `projects/axiom/benches/README.md`. Issue #3 asks for
   this.
4. Compare the median with the budget and with the half-budget headroom target. Mark the result PASS
   or FAIL here. A FAIL opens a follow-up issue.
5. Repeat the same procedure after hot-path changes, such as issues #1, #2 and #11, on the same
   machine.

Full-size and quick-size numbers are never compared with each other.

## How to reproduce

```sh
cd projects/axiom
cargo bench --bench belt_throughput            # full size: use this for the budget
cargo bench --bench belt_throughput -- --quick # smoke size: not comparable with full size
```

The output lines have the form `bench <scenario>.<metric> <value> <unit>`.
