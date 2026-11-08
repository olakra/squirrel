# squirrel

Monorepo for Oscar's projects and their shared documentation.

## Layout

- `projects/<name>/`: one directory per project, each self-contained with its own build and tests.
  - `projects/axiom/`: deterministic 2D factory simulation (Rust core, WASM target, browser harness).
- `docs/`: cross-project documentation.
  - `docs/specs/`: source specifications and master prompts that projects are built from.

## Conventions

- Each project keeps its own `README.md` and `docs/ARCHITECTURE.md`.
- Run project tooling from the project directory (e.g. `cd projects/axiom && cargo test`).
- Keep build output and secrets out of git (see `.gitignore`).
