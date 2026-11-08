# Axiom tech stack: summary

Summary of [ADR 0002: Adopt the Axiom tech stack](adr/0002-tech-stack.md). The ADR holds the options,
evidence and follow-up issues for each layer. Status: **Kept** (no change), **Changed** (current code or
config changes), **Added** (new tool or check), **Deferred** (decided, not built yet).

| Layer | Choice | Status | Reason |
|---|---|---|---|
| Simulation language | Rust, std only | Kept | Integer semantics and typed identifiers fit the integer-state rule, and the tick uses about 1.4% of budget here. |
| Toolchain pin | `rust-toolchain.toml` on a fixed stable, then edition 2024 | Added | A floating `stable` in CI prevents identical rebuilds. |
| Simulation data model | Explicit typed `Vec` pools, no ECS crate | Kept | Explicit index order is easier to review for determinism than ECS query order. |
| WASM bridge | wasm-bindgen, bridge module compiled for wasm32 only | Changed | Typed TypeScript bindings and WebAudio support need it, and the C ABI has no types. |
| Threading | One worker, single-threaded simulation | Changed | The simulation fits the budget on one thread, and Pages cannot set COOP and COEP. |
| Threading upgrade | SharedArrayBuffer and worker pools | Deferred | Adopt only if a measured stress tick on the Deck passes half the budget. |
| Rendering | WebGL2 instanced sprites on OffscreenCanvas | Kept | WebGL2 runs in all four required browsers and in Chromium on Linux. |
| WebGPU and wgpu | Alternative renderers | Deferred | WebGPU is not on Linux yet, and wgpu is the native-path option if the Deck fails. |
| Browser shell | TypeScript, Bun as toolchain, no UI framework | Changed | Generated types guard the WASM boundary, and one framework-free state owner stays in the sim. |
| Static types check | `tsc --noEmit` in CI | Added | Bun does not type-check, so CI needs a separate check. |
| Desktop host | Electron, pinned to supported major 45 | Changed | One Chromium engine across OSes, and the current `^40` pin is out of support. |
| Tauri and native Rust host | Alternative hosts | Deferred | Tauri adds three engines; native Rust is the fallback if the Deck spike fails. |
| Property tests | proptest as a dev-dependency | Added | Belt invariants need generated inputs, and proptest adds no runtime code. |
| Mutation testing | cargo-mutants, nightly, non-blocking | Added | Coverage shows code ran, and mutation shows whether a wrong result would be caught. |
| Test runner | `cargo test` | Kept | The 47 unit tests run in under a second, so nextest adds nothing yet. |
| Test runner upgrade | cargo-nextest | Deferred | Adopt when a test can hang or the suite takes around a minute. |
| Benchmarks | std-only harness | Kept | It prints state hashes, so it also checks determinism. |
| Benchmark upgrade | Criterion | Deferred | Criterion needs stored baselines, which is the decision in roadmap #4. |
| State digest | BLAKE3 over canonical bytes at checkpoints | Changed | The studio spec names BLAKE3, and FNV-1a was a stopgap for roadmap #9. |
| Coverage | cargo-llvm-cov, gates at 90, 90 and 85 percent | Kept | The gate matches requirement R7, but it has not run in CI yet (roadmap #5). |
| Supply chain | cargo-deny; cargo-audit not added | Kept | cargo-deny already checks the RustSec advisory database. |
| Build cache | Swatinem rust-cache; sccache deferred | Kept | sccache pays off only with large dependency trees, and this crate has none. |
| Reproducibility | `--locked`, pinned tools, two-build WASM hash job | Added | Floating tools and an unpinned nightly prevent identical builds. |
| WASM build flags | One flag set for CI, release and Steam | Changed | Today CI and Steam build without threads, and release builds with them. |
| Save files | Hand-written versioned binary with BLAKE3 header; no serde on state | Deferred | Explicit layout keeps saves independent of format-crate versions (roadmap #19). |
| Docs | Markdown, GitHub Mermaid, existing `build.py` | Kept | It needs no installs and already checks links, anchors and Mermaid keywords. |
| Docs upgrade | mdBook | Deferred | Adopt when a player or modding manual needs book navigation and search. |
