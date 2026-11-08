# Contributing

Thanks for working on squirrel. This page covers setup, the branch and PR flow, and
commit messages. The engineering standards are in [docs/engineering/](docs/engineering/).

## Setup

You need:

- Rust stable with `rustfmt`, `clippy`, and `llvm-tools-preview`, plus the
  `wasm32-unknown-unknown` target.
- `cargo-llvm-cov` (coverage gate) and `cargo-deny` (supply-chain gate).
- Nightly Rust, which `projects/axiom/scripts/build-wasm.sh` needs for `build-std`.
- Bun 1.4.4, for the desktop shell and the JavaScript tests (`make js-test`). Install it from bun.sh.
- Node.js 22 or newer. Electron's install script runs on Node, and markdownlint runs through `npx`
  in CI and pre-commit.
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
JavaScript targets, which need Bun and no network: `make js-test` (Bun tests for `web/` and
`desktop/`) and `make js-check` (parses every hand-written `.js` and `.mjs` file). `js-test` is not
part of `make ci`, because CI does not install Bun.

Governance targets, which need no network: `make scripts-test` (runs every script self-test),
`make commit-lint` and `make changelog-check` (compare against `origin/main`), and
`make release-check` (checks the release tag rules against `Cargo.toml`, and tags nothing).

## Branches and pull requests

- `main` is protected. Nobody pushes to it directly. Every change goes through a pull
  request, and every required check must pass before merge. Merges are squash only.
- Branch from `main`. Use `feature/<name>` for new behaviour, `fix/<name>` for a bug fix, and
  `chore/<name>` for docs, pipeline, or maintenance work. For example,
  `feature/inserter-state-machine` or `fix/lane-compression`.
- Keep a pull request under about 400 changed lines. If it is bigger, split it.
- Fill in the pull request template: change type, risk class, rollback plan, determinism impact,
  changelog entry or `no-changelog` reason, and test evidence.
- A change to `projects/axiom/src` or `projects/axiom/Cargo.toml` needs an entry in
  `projects/axiom/CHANGELOG.md` under `[Unreleased]`, or the `no-changelog` label with a reason.
- Review standards are in [docs/engineering/code-review.md](docs/engineering/code-review.md).
  Gate details are in [docs/engineering/quality-gates.md](docs/engineering/quality-gates.md).
  The change lifecycle and risk classes are in
  [docs/engineering/change-management.md](docs/engineering/change-management.md).

## Commit messages

Use [Conventional Commits](https://www.conventionalcommits.org/). The full rules are in
[docs/engineering/commit-conventions.md](docs/engineering/commit-conventions.md).

```text
<type>(<scope>): <short imperative summary>

<optional body: why the change was made, not what the diff shows>
```

- Types: `feat`, `fix`, `perf`, `refactor`, `test`, `docs`, `build`, `ci`, `chore`, `revert`.
- Scope is the project or area, such as `axiom`, `sim`, `ci`, `docs`, or `release`.
- Keep the summary under 72 characters, in the imperative mood.
- A breaking change has `!` after the type and a `BREAKING CHANGE:` footer.
- Example: `fix(axiom): compress back-pressure in one pass`
- The pull request title becomes the commit on `main`, so it must follow the same format.
- `Co-Authored-By` and `Claude-Session` trailers are allowed for AI-assisted work.

## Releases

Releases are cut by the owner from `main`. The steps are in
[docs/engineering/release-process.md](docs/engineering/release-process.md).
Tags are never moved or deleted. See [docs/engineering/tag-protection.md](docs/engineering/tag-protection.md).

## Architecture decisions

Record a decision that affects structure, tooling, or a public contract as an ADR in
[docs/engineering/adr/](docs/engineering/adr/). Copy `0000-template.md` to the next number.
