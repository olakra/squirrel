# Contributing

Thanks for working on squirrel. This page covers setup, the branch and PR flow, and
commit messages. The engineering standards are in [docs/engineering/](docs/engineering/).

## Setup

You need:

- Rust stable with `rustfmt`, `clippy`, and `llvm-tools-preview`, plus the
  `wasm32-unknown-unknown` target.
- `cargo-llvm-cov` (coverage gate) and `cargo-deny` (supply-chain gate).
- Nightly Rust, which `projects/axiom/scripts/build-wasm.sh` needs for `build-std`.
- Node.js 22 or newer (markdownlint runs through `npx`).
- GNU `make`.
- Optional: `pre-commit`, to run the same checks on every commit.

```sh
rustup component add rustfmt clippy llvm-tools-preview
rustup target add wasm32-unknown-unknown
cargo install cargo-llvm-cov --locked
cargo install cargo-deny --locked
pip install pre-commit && pre-commit install
```

## Run the checks locally

```sh
make ci
```

`make ci` runs the same commands as the CI workflow, in the same order: format check,
clippy, tests, coverage gate, bench smoke test, wasm build, cargo-deny, and markdownlint.
Run `make ci` before you push. A green local run should mean a green CI run.

Other targets: `make fmt` (formats code), `make test`, `make cov`, `make bench`, `make wasm`.

## Branches and pull requests

- `main` is protected. Nobody pushes to it directly. Every change goes through a pull
  request, and the CI checks must pass before merge.
- Branch from `main`. Use a short name that says what changes, such as
  `feat/inserter-state-machine` or `fix/lane-compression`.
- Keep a pull request under about 400 changed lines. If it is bigger, split it.
- Fill in the pull request template. A CODEOWNERS reviewer approves each change.
- Review standards are in [docs/engineering/code-review.md](docs/engineering/code-review.md).
  Gate details are in [docs/engineering/quality-gates.md](docs/engineering/quality-gates.md).

## Commit messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```text
<type>(<scope>): <short imperative summary>

<optional body: why the change was made, not what the diff shows>
```

- Types: `feat`, `fix`, `perf`, `refactor`, `test`, `docs`, `build`, `ci`, `chore`.
- Scope is the project or area, such as `axiom`, `sim`, `ci`, or `docs`.
- Keep the summary under 72 characters, in the imperative mood.
- Example: `fix(axiom): compress back-pressure in one pass`

## Architecture decisions

Record a decision that affects structure, tooling, or a public contract as an ADR in
[docs/engineering/adr/](docs/engineering/adr/). Copy `0000-template.md` to the next number.
