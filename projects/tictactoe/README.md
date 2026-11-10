# Tic-tac-toe

A tic-tac-toe game that runs in the browser. The rules are plain Rust. A thin wrapper builds them to
WebAssembly, and a small HTML and JavaScript page draws the board.

The page is built for keyboard and screen-reader players. Each cell is a button with a label that
names its row, column, and mark. A polite live region announces each turn, the result, and any
refused move. Every visible string comes from a translation file. A line is drawn across the
winning cells, and Reset clears the board and the line.

## Layout

| Path | What it holds |
|---|---|
| `Cargo.toml` | The Cargo workspace for `core` and `web`. |
| `core/` | `tictactoe-core`: the rules. Board, turns, wins, draws, and refused moves. No dependencies, no I/O. Tests are in `core/tests/`. |
| `web/` | `tictactoe-web`: the `TicTacToe` class exported to JavaScript with `wasm-bindgen`. It only converts types. `build.sh` builds the browser bundle. |
| `web/www/` | The page: `index.html`, `main.js`, `styles.css`, the pure modules in `lib/`, and the messages in `i18n/en.json`. `build.sh` writes the wasm module and its glue to `www/pkg/`, which is not committed. |
| `web/test/` | Node tests for the pure JavaScript modules and the page markup. |

## Build and test

Run these from the repository root.

```sh
make tictactoe-check   # cargo fmt check, clippy, and tests for both crates
make tictactoe-web     # builds the browser bundle into web/www/pkg
```

`make tictactoe-web` needs the `wasm32-unknown-unknown` target and `wasm-bindgen-cli` 0.2.100. The CLI
version must equal the exact pin of `wasm-bindgen` in `web/Cargo.toml`, or the glue does not match the
`.wasm` file. Install it with `cargo install wasm-bindgen-cli --version 0.2.100 --locked`.

To play, serve `web/www` with any static file server, then open the printed address:

```sh
python -m http.server -d projects/tictactoe/web/www
```

Opening `index.html` from disk does not work, because browsers do not load wasm modules from `file://`.

The JavaScript tests need Node.js 22 or newer and no network:

```sh
cd projects/tictactoe/web && node --test test/*.test.js
```

CI runs the same Rust checks and the bundle build in the `tictactoe` job, and keeps the built site as
the `tictactoe-site` artifact.

## Versioning and release

Tic-tac-toe is versioned on its own, with SemVer. The version is in `Cargo.toml` and is mirrored in
`web/package.json`. Changes are recorded in [CHANGELOG.md](CHANGELOG.md) under `[Unreleased]`.

Releases are tagged `tictactoe-vX.Y.Z`. The project has no release of its own yet. Its first will be
`tictactoe-v0.1.0`. The flow is the same as for every project: `make release-prepare PROJECT=tictactoe
VERSION=0.1.0`, a release PR, then the owner's `make release-tag` and `make release-push`. See
[CONTRIBUTING.md](../../CONTRIBUTING.md#releases) and
[docs/engineering/release-process.md](../../docs/engineering/release-process.md).

## Studio docs

- [Studio personas](../../docs/studio/personas.md): the six roles, their decisions, and their handoffs.
- [Art direction](../../docs/studio/art-direction.md): colour tokens with contrast ratios, type, spacing,
  motion, and i18n rules.
- [Icons](../../docs/studio/icons.md): the icon set and the grid rules each icon follows.
