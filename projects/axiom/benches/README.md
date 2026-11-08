# Belt throughput benchmarks

A std-only benchmark suite for the belt simulation. It uses a custom harness
(`harness = false`), so it builds and runs offline with no crates.

## Running

```sh
cargo bench --bench belt_throughput            # full sizes
cargo bench --bench belt_throughput -- --quick  # ~10x smaller smoke run for CI
```

Run from the crate root. Each run prints plain text lines, and nothing else
goes to stdout:

```text
bench <scenario>.<metric> <value> <unit>
```

Each scenario prints three metrics:

| Metric | Unit | Meaning |
|---|---|---|
| `ns_per_tick` | `ns/tick` | Median wall time per tick across the measured runs. For `rear_push` one tick is one push, advance, pop cycle. |
| `items_per_sec` | `items/s` | Item moves per second: `items_per_tick * 1e9 / ns_per_tick`. |
| `state_hash` | `fnv1a64` | FNV-1a 64 fingerprint of the state after the run. |

Example:

```text
bench rear_push.ns_per_tick 357.3 ns/tick
bench rear_push.items_per_sec 2799069 items/s
bench rear_push.state_hash 0x3a569a910ca391a3 fnv1a64
```

## Scenarios

| Scenario | Full size | `--quick` | What it measures |
|---|---|---|---|
| `stress_100k_items_10k_segments` | 10,000 lanes, 100,000 items, 1,000 ticks | 1,000 lanes, 10,000 items, 100 ticks | Ticking a large, densely packed belt network with the exit open, so items move and drain at the tail. |
| `blocked_backpressure` | same layout | same layout | The same layout with the exit closed, so every queue is compressed and stationary. Measures the cost of advancing packed lanes that cannot move. |
| `rear_push` | 1 lane, 1,000,000 cycles | 1 lane, 100,000 cycles | The hot path: `try_push_rear`, `advance` and `pop_front_if_ready` on one busy lane. |

Layout: lanes are grouped into chains of 100 lanes, each lane 60 units long at
speed 2. Setup is untimed. The layout is filled through `try_feed` with the
exit closed, then left to settle for 6,000 ticks so that every chain tail is
backed up. Each timed run works on a clone of that settled layout.

The `stress` and `blocked` runs have a sanity check. The blocked run asserts
that its state is unchanged by the timed ticks. If it moves, the bench panics,
because the layout was not settled.

Every measured run also checks that the state hash is identical across runs.
A mismatch panics, which would mean the simulation is not deterministic.

`items_per_sec` counts item moves. For the belt scenarios the item count is the
average of the counts at the start and end of the timed window. That is an
approximation when items leave through the exit during the run.

## Comparing two runs

Run the suite on a baseline and on a change, on the same machine, with the same
mode (`--quick` against `--quick`, full against full). Quick and full numbers
are not comparable.

```sh
cargo bench --bench belt_throughput > before.txt
# ...make the change...
cargo bench --bench belt_throughput > after.txt

# 1. Behavior check. For a pure speed change the hashes must match.
#    A hash change means the simulated state changed, not just its speed.
diff <(grep state_hash before.txt) <(grep state_hash after.txt) && echo "hashes identical"

# 2. Speed ratios. Below 1.00x on ns_per_tick is faster.
join <(grep -E 'ns_per_tick|items_per_sec' before.txt | awk '{print $2, $3}' | sort) \
     <(grep -E 'ns_per_tick|items_per_sec' after.txt  | awk '{print $2, $3}' | sort) \
  | awk '{printf "%-50s %s -> %s (%.2fx)\n", $1, $2, $3, $3/$2}'
```

Single runs are noisy. On a shared machine `rear_push` varied by about 25%
between back-to-back runs with no code change. Repeat each run three or more
times, compare medians, and only trust differences larger than the noise.

## CI

Use `--quick` for smoke runs. Parse lines that start with `bench` and split on spaces.
The name is `<scenario>.<metric>`, the value is field 3, and the unit is
field 4.
