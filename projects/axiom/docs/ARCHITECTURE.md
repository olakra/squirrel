# Project Axiom: Architecture (Milestone 0)

## Status

| Area | State |
|---|---|
| Interval-queue belt lane + chain | Implemented, tested |
| Fixed-timestep world + state hash | Implemented, tested |
| 1,000-tick determinism regression | Implemented (hash parity across runs) |
| Splitters, mergers, inserters | Not started |
| WASM bindings (`wasm-bindgen`) | Not started (needs registry access) |
| Web Worker simulation + SharedArrayBuffer | Harness only, no WASM yet |
| Rendering (instanced sprites, WebGL2) | Placeholder canvas only |
| Criterion benchmarks | Not started (needs registry access) |

## Constraints in force

- **Integer-only simulation.** No floats in simulation state.
- **Deterministic ticks.** Fixed 60 Hz step, independent of render rate.
- **No allocation in the tick path.** Lanes reserve capacity at construction.
  The one exception is `try_push_rear`, which may memmove but never grows
  the buffer past its reserved capacity. This is a known hot spot.
- **Dependency-free core.** The crate builds with the standard library only,
  so `cargo test` runs offline.

## Belt model

Each `BeltLane` holds ascending `(item, offset)` pairs.

- `items[0]` is the rear (offset near 0). `items[last]` is the front.
- Advance front to back. Each item moves `speed` units, capped at
  `next.offset - ITEM_SPACING`. The front item is capped at
  `length - ITEM_SPACING`.
- A front item at the cap is ready to exit. If downstream accepts it, it
  moves on. If not, the queue compresses backward in one pass.

`BeltChain` links lanes. Each tick processes downstream lanes first, so a
lane frees room before its upstream neighbour hands items over.

## Known gaps and decisions to revisit

1. **Rear insertion is O(N).** Switch to a ring buffer before adding
   200k-item stress tests.
2. **Spec conflict on hashing.** The spec names both FNV-1a and Blake3. FNV-1a
   is used now (no dependency). Swap in Blake3 when dependencies are available.
3. **Spec conflict on fixed-point.** The spec allows `I32F32` or discrete
   integers. Discrete integers are used, which removes the `fixed` dependency.
4. **Lane-end semantics.** A blocked front item rests at `length - SPACING`
   rather than at `length`, to keep the visual queue spaced.
5. **Chain order.** Chains are independent in `World::step`. Cross-chain
   interactions (inserters, splitters) will need a fixed, documented order.

## Next milestones

1. Splitter (round-robin, deterministic) with lane-balance tests.
2. Inserter state machine with reservation.
3. Chunk grid (32x32) with sleep/wake.
4. `wasm-bindgen` build script and a browser test.
5. Ring-buffer lanes and a 100k-item Criterion benchmark.
