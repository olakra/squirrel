# squirrel

[![CI](https://github.com/olakra/squirrel/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/olakra/squirrel/actions/workflows/ci.yml)
[![Pages](https://github.com/olakra/squirrel/actions/workflows/pages.yml/badge.svg?branch=main)](https://github.com/olakra/squirrel/actions/workflows/pages.yml)
[![Release](https://img.shields.io/github/v/release/olakra/squirrel?include_prereleases&sort=semver)](https://github.com/olakra/squirrel/releases)
![License](https://img.shields.io/badge/license-proprietary-red)
![Rust](https://img.shields.io/badge/rust-stable-orange?logo=rust)
![Bun](https://img.shields.io/badge/bun-1.4.4-black?logo=bun)
![Conventional Commits](https://img.shields.io/badge/commits-conventional-fe5196?logo=conventionalcommits&logoColor=white)

Monorepo for Oscar's projects and their shared engineering documentation.

## Projects

| Project | What it is | Status |
|---|---|---|
| [`projects/axiom`](projects/axiom) | Deterministic 2D factory simulation: Rust core, WASM browser build, desktop shell, Steam packaging | Milestone 1 (simulation core) |

## Repository layout

- `projects/<name>/`: one directory per project. Each is self-contained with its own build, tests, and README.
- `docs/`: cross-project documentation.
  - `docs/engineering/`: engineering principles, quality gates, change and release management, and ADRs.
  - `docs/specs/`: source specifications that projects are built from.
  - `docs/roadmap.md`: milestones and their dependencies.
- `scripts/ci/`: repository-level CI checks (commit messages, changelog, tags, branch protection).
- `.github/`: workflows, rulesets, issue and pull request templates, CODEOWNERS.

## Working in this repo

- Start with [CONTRIBUTING.md](CONTRIBUTING.md). It covers setup, branches, commit messages, and pull requests.
- Run the full local check with `make ci`. It runs the same gates as CI.
- Track bugs and features in [GitHub Issues](https://github.com/olakra/squirrel/issues). The roadmap lives in [docs/roadmap.md](docs/roadmap.md).
- Report security issues privately. See [SECURITY.md](SECURITY.md).

## Quality bar

- Formatting, clippy with warnings denied, tests on Linux, macOS, and Windows,
  a coverage gate, a benchmark smoke run, and dependency audit run on every pull request.
- Changes are merged only through reviewed pull requests with linear history.
- Releases are tagged `vX.Y.Z`, built in CI, and attested with build provenance. See [docs/engineering/release-process.md](docs/engineering/release-process.md).

## License

Proprietary. All rights reserved. See [LICENSE](LICENSE).
