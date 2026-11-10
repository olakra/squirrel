# CLAUDE.md

Rules for AI agents working in this repo: Claude Code CLI, desktop, IDE, and web. Agents follow the same rules
as people, plus the limits below.

Read first: [CONTRIBUTING.md](CONTRIBUTING.md) and
[docs/engineering/branching-strategy.md](docs/engineering/branching-strategy.md).

## Branches

- Always work on a branch. Create it with `make branch TYPE=<type> NAME=<short-name>`.
- Work branches are `<type>/<short-name>`, with type in `feature`, `fix`, `chore`, `docs`, `ci`, `build`,
  `perf`, `refactor`, `test`. Lowercase, `a-z 0-9 . _ -` only. Check with `make branch-check`.
- Claude Code cloud sessions may use `claude/**` branches.
- Never commit or push to `main` or to `support/*`. They change only through squash-merged PRs.

## Commits and pull requests

- Conventional Commits. Subject 72 characters or fewer, imperative mood. Rules:
  [docs/engineering/commit-conventions.md](docs/engineering/commit-conventions.md).
- The PR title is the squash commit subject on `main`. It follows the same format.
- Before you push, run `make ci`. Also run `make js-test` when JavaScript changed.
- A change to a project's source needs an entry under `[Unreleased]` in that project's `CHANGELOG.md`
  (`projects/axiom/CHANGELOG.md` or `projects/tictactoe/CHANGELOG.md`). Otherwise ask for the
  `no-changelog` label and give the reason in the PR.
- Keep a PR under about 400 changed lines. Split bigger work.
- Fill in every section of the PR template.
- Record an architectural decision as an ADR in [docs/engineering/adr/](docs/engineering/adr/).
- Never merge your own PR unless the owner asks you to.

## Never do these

- Never create, push, move, or delete a tag.
- Never create, edit, or delete a GitHub release.
- Never run `make release-tag`, `make release-push`, `make hotfix-start`, or `make github-protect`. They are
  owner only.
- Never force-push.
- Never bypass hooks or signing: no `--no-verify`, no `--no-gpg-sign`, no `-c commit.gpgsign=false`.
- Never change branch protection, rulesets, environments, or repository settings.
- Never commit secrets, tokens, or credentials.

You may run `make release-prepare`, `make release-check`, and `make release-pr`, and open the release PR.
The owner merges it, tags, and pushes the tag.

## Axiom simulation rules

From [projects/axiom/docs/ARCHITECTURE.md](projects/axiom/docs/ARCHITECTURE.md):

- Integer-only simulation. No floats in simulation state.
- Fixed 60 Hz tick, independent of the render rate.
- No allocation in the tick path. Lanes reserve capacity at construction.
- Dependency-free core. The crate builds with the standard library only.
- Any change to simulation behaviour needs a determinism test (fixed-tick state hash). If the hash or a golden
  value changes, say so in the PR's determinism section.

## Repo layout

| Path | What it is |
|---|---|
| `projects/axiom/` | Factory simulation: Rust core, WASM build, web, desktop shell, Steam packaging |
| `projects/tictactoe/` | Tic-tac-toe: Rust rules core (`core/`) and WebAssembly front end (`web/`) |
| `docs/engineering/` | Standards, gates, branching, release process, ADRs |
| `docs/studio/` | Studio roles, art direction, icons |
| `docs/roadmap.md`, `docs/releases/` | Milestones and release notes |
| `scripts/ci/` | CI checks: commit messages, changelog, branch names, tags |
| `scripts/release/` | Release scripts and the project registry `projects.sh` |
| `.github/` | Workflows, rulesets, templates, CODEOWNERS |

## Key commands

| Command | What it does |
|---|---|
| `make help` | Lists every target |
| `make branch TYPE=feature NAME=x` | Creates `feature/x` from `origin/main` |
| `make ci` | Every local gate, in CI order |
| `make js-test` | Bun tests for axiom's web and desktop code |
| `make tictactoe-check` | fmt, clippy, and tests for tic-tac-toe |
| `make scripts-test` | Script self-tests |
| `make commit-lint`, `make changelog-check` | Check the branch against `origin/main` |
| `make versions` | Each project's version and latest tag |
| `make release-prepare PROJECT=p VERSION=x.y.z` | Creates the release branch and commit. Pushes nothing |
| `make release-check PROJECT=p` | Checks that tag, version, and changelog agree. Tags nothing |

Tic-tac-toe JavaScript tests: `cd projects/tictactoe/web && node --test test/*.test.js`.
