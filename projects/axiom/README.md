# Project Axiom

A deterministic 2D factory simulation. The core is Rust with no dependencies. A browser harness in
`web/` runs the WASM build. A desktop shell and Steam packaging sit on top of the browser build.

## Layout

| Path | Purpose |
|---|---|
| `src/` | Simulation core and the WASM export layer. |
| `scripts/build-wasm.sh` | Builds `web/axiom.wasm`. |
| `scripts/serve.py` | Serves `web/` with the COOP and COEP headers that `SharedArrayBuffer` needs. |
| `web/` | Browser harness: `index.html`, `main.js`, `worker.js`, `input.js`, `styles/deck.css`. |
| `desktop/` | Electron shell that runs `web/` as a native app. See `desktop/README.md`. |
| `steam/` | Steamworks templates, the Deck Verified checklist, and the owner steps. See `steam/README.md`. |
| `docs/ARCHITECTURE.md` | Simulation architecture and known gaps. |

## Browser build

```bash
cd projects/axiom
bash scripts/build-wasm.sh     # writes web/axiom.wasm
python3 scripts/serve.py       # http://127.0.0.1:8080
```

The page still loads without `axiom.wasm`. It shows a status line saying the simulation is not loaded.

### Controls

| Action | Controller (Xbox layout) | Keyboard |
|---|---|---|
| Move cursor | Left stick or D-pad | Arrows or WASD |
| Confirm | A | Enter or Space |
| Cancel | B | Escape |
| Show or hide hints | Start | H |

Input is translated into intents in `web/input.js`. `web/main.js` applies the cursor and hints, and
forwards `confirm` and `cancel` to the worker. The worker does not handle them yet. Simulation state is
changed only by the worker.

## Desktop and Steam

Desktop packaging and Steam uploads come after the browser build works. Both are documented in their
own folders. The Steam upload runs in `.github/workflows/steam-build.yml` on version tags.

## Checks

```bash
cd projects/axiom
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
```

The repository `Makefile` runs the full set as `make ci`.
