Copy and paste the master prompt below directly into **Claude**, **DeepSeek**, or **Grok**. It is structured with explicit architectural constraints, memory layouts, WebAssembly threading flags, and test requirements to guide an LLM to generate production-ready code.

---

```markdown
# SYSTEM ARCHITECTURE & MASTER PROMPT: DETERMINISTIC 2D FACTORY SIMULATION (RUST + WASM)

You are an expert principal game engine architect and systems programmer specializing in Rust, WebAssembly, and high-performance deterministic simulation engines (such as Factorio).

Your objective is to design, implement, and fully test an ultra-high-performance 2D factory simulation game running purely in the web browser via Rust compiled to WebAssembly (`wasm32-unknown-unknown`).

---

## 1. PERFORMANCE & HARDWARE CONSTRAINTS (ZERO-GPU DEPENDENCY)
1. **Low-Spec Hardware Target**: The engine must sustain 60 UPS (Updates Per Second) and 60 FPS on low-end hardware (e.g., standard consumer laptops with integrated graphics). Do not rely on compute shaders or heavy GPU passes.
2. **Deterministic Tick Engine (Fixed Timestep)**: Decouple simulation tick rate (60 Hz fixed) from rendering frame rate. All simulation logic must be 100% deterministic (no floating-point non-determinism; use fixed-point arithmetic `I32F32` or discrete integer math for simulation states).
3. **Data-Oriented Design (DOD / ECS)**:
   - Use strict Structure-of-Arrays (SoA) layout or an archetype/dense-array ECS (such as `hecs` or raw contiguous arena slices).
   - Zero heap allocations inside the hot tick loop. All belt segments, inserters, and items must reside in cache-friendly, pre-allocated contiguous memory pools (`Vec` / arena slots).
4. **Browser Hardware Maximization**:
   - Utilize Web Workers backed by `SharedArrayBuffer` via `wasm-bindgen-rayon` or standard Web Worker pool with shared linear memory.
   - Utilize WebAssembly SIMD128 (`target-feature=+simd128`) for batch processing item positions, conveyor updates, and spatial queries.
   - Offload the simulation loop into dedicated Web Worker(s). Use `OffscreenCanvas` for rendering (WebGL2 or 2D batch sprite renderer) so the main DOM thread remains 100% unblocked for UI and input handling.

---

## 2. CORE FACTORY SIMULATION MECHANICS
Implement the foundational mechanics of a Factorio-style game:
1. **Conveyor Belt System (Compound Array / Interval Queues)**:
   - Do NOT simulate each item as an independent physics actor.
   - Represent continuous belt chains as serialized lanes/intervals: track belt lane capacity, item IDs, lane offsets, and front/back congestion states.
   - Belts push chunks of items forward in bulk; items back up seamlessly when congested.
2. **Inserter Mechanics**:
   - Discrete state machine: `SearchingSource`, `PickingUp`, `RotatingToTarget`, `Dropping`, `Resetting`.
   - Inventory reservation to prevent race conditions during insertion/extraction across threaded workers.
3. **Chunked Spatial Grid & Map Management**:
   - Chunk size: 32x32 tiles per chunk.
   - Dynamic chunk activation: only tick chunks that contain active machinery or adjacent flowing belts (sleep inactive/static chunks).
4. **Rendering Pipeline**:
   - Batched multi-layer 2D instanced sprite renderer using WebGL2 (or lightweight `wgpu` targeting WebGL2/WebGPU fallback).
   - Single draw call per texture atlas with instanced vertex buffers (pos_x, pos_y, tile_id, rotation).

---

## 3. COMPLIANCE, SECURITY, & BROWSER GOVERNANCE
1. **COOP/COEP Headers**: Explicitly define and handle requirements for Cross-Origin-Opener-Policy (`same-origin`) and Cross-Origin-Embedder-Policy (`require-corp`) to permit `SharedArrayBuffer`.
2. **Graceful Fallbacks**: Provide runtime feature detection. If `SharedArrayBuffer` is blocked by browser policy, fall back cleanly to a single-threaded WASM loop with clear logging.
3. **Memory Safety & Sandbox Bounds**:
   - Strictly prohibit undefined behavior. Any `unsafe` block (e.g., raw pointer slices for SIMD or zero-copy worker views) must include an explicit `# Safety` contract document.
   - WASM heap growth management: pre-allocate memory chunks to prevent frequent `memory.grow` execution spikes.

---

## 4. CODE QUALITY & TEST SUITE
1. **Unit & Determinism Tests (`cargo test --target x86_64-...`)**:
   - Test belt item push, compression, splitters, and lane deadlocks.
   - 1,000-tick determinism regression test: Run a predefined layout for 1,000 ticks, hash the state using FNV-1a or Blake3, and verify the hash matches across runs.
2. **WASM Browser Tests (`wasm-bindgen-test`)**:
   - Integration tests executing directly in headless Chromium/Firefox.
3. **Benchmarks & Stress Test**:
   - A Criterion/micro-benchmark suite stressing 100,000 items on 10,000 active belt segments.

---

## 5. DELIVERABLE SPECIFICATION
Provide a fully functional, production-structured Rust project including:
1. **`Cargo.toml`**: Configured with proper dependencies (`wasm-bindgen`, `web-sys`, `fixed`, `bytemuck`, `parking_lot`, etc.), LTO flags, and release profiles for minimal binary footprint and maximum speed.
2. **Build Configuration (`.cargo/config.toml` & build script)**: Target features (`+atomics`, `+bulk-memory`, `+simd128`) and `RUSTFLAGS`.
3. **`src/lib.rs` & Module Hierarchy**:
   - `core/`: Fixed-point math, coordinate types, entity IDs, ring buffers.
   - `simulation/`: Belt lines, item queues, inserter state machine, chunk grid.
   - `threads/`: Worker pool setup, task partitioning, double-buffered frame state.
   - `render/`: Instanced sprite buffer generation and WebGL2 bindings.
4. **Comprehensive Unit & Determinism Test Code**: Ready to execute via `cargo test`.
5. **Web Harness (`index.html` + `worker.js` + `main.js`)**: Correctly setting up `OffscreenCanvas`, worker dispatch, and canvas resize listeners.

Start by delivering the architecture overview, project layout, and the complete implementations for the core deterministic simulation data structures and tests.

```

---

### Tips for Best Execution

* **Claude (Opus/Sonnet):** Best at handling complete, idiomatic Rust type systems and drafting the intricate `unsafe`-safe boundary code for zero-copy memory transfers between the worker and the renderer.
* **DeepSeek (V3 / R1):** Exceptional at algorithmic optimization—specifically asking it to optimize the belt simulation into compressed interval ranges (similar to Factorio's actual belt line algorithm).
* **Grok:** Useful for generating the glue code, deployment configs (COOP/COEP headers with Vite/Nginx), and build automation scripts.