# ADR 0001: Monorepo layout

- **Status:** Accepted
- **Date:** 2026-11-08
- **Deciders:** repository owner (@olakra)

## Context

The squirrel repository holds more than one project, plus specifications and documentation that
apply to all of them. Today it holds one project, `projects/axiom`, a Rust simulation with a
WebAssembly target. Each project has its own build, tests, and toolchain. Some documents describe
the whole repo, such as the source specs and these engineering standards.

We need one place for the CI definition, review rules, and shared docs, without forcing projects
to share a build system.

Options considered:

1. **One repo per project.** Clean isolation. Shared standards get copied and drift apart.
2. **One Cargo workspace at the root.** Shares one lockfile across languages and projects. It does
   not fit a polyglot repo, and Axiom's `cdylib` and wasm flags would apply to unrelated projects.
3. **A monorepo with one directory per project, each self-contained, and shared docs at the root.**

## Decision

We will use a single monorepo with this layout:

- `projects/<name>/`: one directory per project. Each is self-contained with its own build file,
  tests, `README.md`, and `docs/ARCHITECTURE.md`. There is no root Cargo workspace.
- `docs/`: documentation that spans projects. `docs/specs/` holds source specifications.
  `docs/engineering/` holds the standards in this folder and the ADRs.
- Repo-level governance at the root: `.github/` (CI, release, review templates), `Makefile`
  (`make ci`), `CONTRIBUTING.md`, and lint and hook configuration.
- CI jobs address each project by path. Each project's cargo steps set `working-directory` from an
  environment variable in `.github/workflows/ci.yml`.

## Consequences

- Each project builds and tests on its own. A change in one project does not rebuild another.
- Standards are written once, in `docs/engineering/`, and apply to every project.
- The CI workflow is project-aware. Adding a second Rust project means adding its jobs, or moving
  the crate path into a matrix. Path filters can cut CI time once there are several projects.
- Projects cannot share a lockfile or a dependency version without an explicit decision. That is
  intended.
- Any repo-wide tool (markdownlint, pre-commit) must exclude build output and vendored content
  explicitly. See `.markdownlint-cli2.jsonc`.
