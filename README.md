# squirrel

[![CI](https://github.com/olakra/squirrel/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/olakra/squirrel/actions/workflows/ci.yml)
[![Pages](https://github.com/olakra/squirrel/actions/workflows/pages.yml/badge.svg?branch=main)](https://github.com/olakra/squirrel/actions/workflows/pages.yml)
[![Release](https://img.shields.io/github/v/release/olakra/squirrel?include_prereleases&sort=semver)](https://github.com/olakra/squirrel/releases)
![License](https://img.shields.io/badge/license-proprietary-red)
![Rust](https://img.shields.io/badge/rust-stable-orange?logo=rust)
![Bun](https://img.shields.io/badge/bun-1.4.4-black?logo=bun)
![Conventional Commits](https://img.shields.io/badge/commits-conventional-fe5196?logo=conventionalcommits&logoColor=white)

The monorepo of a small indie game studio: the games, the tools they share, and the engineering
docs that apply to all of them.

## Projects

| Project | What it is | Status |
|---|---|---|
| [`projects/axiom`](projects/axiom) | Deterministic 2D factory simulation: Rust core, WASM browser build, desktop shell, Steam packaging | Milestone 1 (simulation core), released as `v0.1.2` |
| [`projects/tictactoe`](projects/tictactoe) | Tic-tac-toe web game: Rust rules core and a WebAssembly front end | Playable web build. First release will be `tictactoe-v0.1.0` |

## Repository layout

- `projects/<name>/`: one directory per project. Each is self-contained with its own build, tests, README,
  and changelog.
- `docs/`: cross-project documentation.
  - `docs/engineering/`: engineering principles, quality gates, branching, change and release management,
    and ADRs.
  - `docs/studio/`: studio roles, art direction, and icon rules.
  - `docs/specs/`: source specifications that projects are built from.
  - `docs/releases/`: release notes.
  - `docs/roadmap.md`: milestones and their dependencies.
- `scripts/ci/`: repository-level CI checks (commit messages, changelog, branch names, tags, branch protection).
- `scripts/release/`: release scripts and the project registry (`projects.sh`) they all read.
- `.github/`: workflows, rulesets, issue and pull request templates, CODEOWNERS.
- `CLAUDE.md`: rules for AI agents working in this repo.

## Working in this repo

- Start with [CONTRIBUTING.md](CONTRIBUTING.md). It covers setup, branches, commit messages, and pull requests.
- Run `make help` to list every make target.
- Start work with `make branch TYPE=feature NAME=<short-name>`. Branch names and the release and hotfix
  branches are in [docs/engineering/branching-strategy.md](docs/engineering/branching-strategy.md).
- Run the full local check with `make ci`. It runs the same gates as CI.
- Track bugs and features in [GitHub Issues](https://github.com/olakra/squirrel/issues). The roadmap lives in
  [docs/roadmap.md](docs/roadmap.md).
- Report security issues privately. See [SECURITY.md](SECURITY.md).

## Quality bar

- Formatting, clippy with warnings denied, tests on Linux, macOS, and Windows,
  a coverage gate, a benchmark smoke run, and dependency audit run on every pull request.
- `main` changes only through squash-merged pull requests, after every required check passes.
  History is linear. Nobody pushes to `main` directly, admins included.
- Commits on `main` are signed.
- Each project is released on its own, with a tag `<project>-vX.Y.Z` such as `axiom-v0.2.0`. Releases are
  built in CI and attested with build provenance. See
  [docs/engineering/release-process.md](docs/engineering/release-process.md).

## License

Proprietary. All rights reserved. See [LICENSE](LICENSE).
