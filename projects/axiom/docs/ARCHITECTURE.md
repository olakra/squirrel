# Project Axiom: Architecture (Milestone 1)

## Status

| Area | State |
|---|---|
| Ring-buffer belt lane + chain (O(1) rear insert and front pop) | Implemented, tested (incl. differential test) |
| Splitter (deterministic round-robin, blocked-output routing) | Implemented, tested |
| Inserter state machine with source reservation | Implemented, tested |
| Chunk grid (32x32, sleep/wake, sorted iteration) | Implemented, tested; integrated into `World`: only active chunks tick, in sorted order (issue #1) |
| Fixed-timestep world + FNV-1a state hash | Implemented, tested (1,000-tick determinism regression) |
| Splitters and inserters wired into `World` | Implemented, tested: fixed four-phase tick order (inserter pick, chain advance, splitter route, inserter drop), hashed into state (issue #11) |
| WASM C-ABI exports (`src/wasm.rs`) + build script | Implemented; `wasm32` build verified in CI only |
| Web Worker simulation + SharedArrayBuffer | Worker runs the WASM sim at 60 Hz; `F` feeds an item. Browser check pending (issue #14) |
| Rendering (instanced sprites, WebGL2) | Items drawn from a per-frame WASM snapshot (`axiom_snapshot_items`). Browser check pending (issue #13) |
| Benchmarks (std-only `belt_throughput`) | Implemented; regression gate is a review rule (issue #4) |
| Desktop shell (Electron) and Steam packaging | Implemented; not yet built on Steam (issues #15–#18) |

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
   is kept for 0.1.1 (no dependency); see [ADR 0005](../../../docs/engineering/adr/0005-state-hash.md).
   The switch to Blake3 is one release, with a Breaking changelog entry, once a dependency is allowed.
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
