# Contributing

Thanks for working on squirrel. This page covers setup, the branch and PR flow, commit
messages, and releases. The engineering standards are in [docs/engineering/](docs/engineering/).
AI agents also follow [CLAUDE.md](CLAUDE.md).

## Setup

You need:

- Rust stable with `rustfmt`, `clippy`, and `llvm-tools-preview`, plus the
  `wasm32-unknown-unknown` target.
- `cargo-llvm-cov` (coverage gate) and `cargo-deny` (supply-chain gate).
- Nightly Rust, which `projects/axiom/scripts/build-wasm.sh` needs for `build-std`.
- `wasm-bindgen-cli` 0.2.100, for `make tictactoe-web`. The version must equal the pin in
  `projects/tictactoe/web/Cargo.toml`.
- Bun 1.4.4, for the desktop shell and the JavaScript tests (`make js-test`). Install it from bun.sh.
- Node.js 22 or newer. Electron's install script runs on Node, markdownlint runs through `npx`
  in CI and pre-commit, and the tic-tac-toe JavaScript tests run on `node --test`.
- GNU `make`.
- SSH commit signing. It is required for local work, because `main` accepts only signed commits.
  The steps are in [docs/engineering/claude-code-setup.md](docs/engineering/claude-code-setup.md#signed-commits).
- Optional: `pre-commit`, to run the same checks on every commit.

```sh
rustup component add rustfmt clippy llvm-tools-preview
rustup target add wasm32-unknown-unknown
cargo install cargo-llvm-cov --locked
cargo install cargo-deny --locked
cargo install wasm-bindgen-cli --version 0.2.100 --locked
pip install pre-commit && pre-commit install
```

## Run the checks locally

```sh
make ci
```

`make ci` runs the same commands as the CI workflow, in the same order: format check,
clippy, tests, coverage gate, bench smoke test, wasm build, the tic-tac-toe checks, cargo-deny,
and markdownlint. Run `make ci` before you push. A green local run should mean a green CI run.

Run `make help` to list every target. The main ones:

| Target | What it does |
|---|---|
| `make ci` | Every local gate, in CI order. |
| `make fmt`, `make test`, `make cov`, `make bench`, `make wasm` | Format, test, coverage, benchmark, and wasm build for axiom. |
| `make tictactoe-check` | Format check, clippy, and tests for `projects/tictactoe`. Part of `make ci`. |
| `make tictactoe-web` | Builds the tic-tac-toe browser bundle. Needs `wasm-bindgen-cli` 0.2.100. |
| `make js-test`, `make js-check` | Bun tests for axiom's `web/` and `desktop/`, and a parse check of every hand-written `.js` and `.mjs` file. Need Bun, no network. |
| `make scripts-test` | Runs every script self-test. |
| `make commit-lint`, `make changelog-check` | Compare your branch against `origin/main`. |
| `make branch`, `make branch-check` | Create a work branch, or check the current branch name. |
| `make versions` | Each project's version and latest tag. |
| `make release-prepare`, `make release-pr`, `make release-check` | Prepare a release branch, open its PR, and check that tag, version, and changelog agree. |
| `make release-tag`, `make release-push`, `make hotfix-start` | Tag and push a release, or start a hotfix. Owner only. |
| `make github-protect-dry`, `make github-protect` | Print or apply the rulesets, merge settings, and `production` environment. Owner only. |

`js-test` is not part of `make ci`, because CI does not install Bun.

## Branches and pull requests

- `main` is protected. Nobody pushes to it directly, admins included. Every change goes through a
  pull request, and every required check must pass before merge. Merges are squash only.
- Start a branch with `make branch TYPE=feature NAME=belt-sleep`. It fetches, then creates
  `feature/belt-sleep` from `origin/main`.
- Work branches are `<type>/<short-name>`. The type is one of `feature`, `fix`, `chore`, `docs`, `ci`,
  `build`, `perf`, `refactor`, or `test`. Names are lowercase and use only `a-z`, `0-9`, `.`, `_`, and `-`.
- Release branches (`release/<project>-vX.Y.Z`), hotfix branches (`hotfix/<short-name>`), and support
  branches (`support/<project>-vX.Y`) are made by the release targets, not by hand.
- The `branch-name` CI job checks the name. Run `make branch-check` to check it locally.
- The full model, with the reasons, is in
  [docs/engineering/branching-strategy.md](docs/engineering/branching-strategy.md).
- Keep a pull request under about 400 changed lines. If it is bigger, split it.
- Fill in the pull request template: change type, risk class, rollback plan, determinism impact,
  changelog entry or `no-changelog` reason, and test evidence.
- A change to a project's watched source paths needs an entry in that project's `CHANGELOG.md` under
  `[Unreleased]`, or the `no-changelog` label with a reason. Axiom's changelog is
  `projects/axiom/CHANGELOG.md`. Tic-tac-toe's is `projects/tictactoe/CHANGELOG.md`. Repository-wide
  changes, such as docs and CI, need no project changelog.
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
- Scope is the project or area, such as `axiom`, `tictactoe`, `sim`, `ci`, `docs`, or `release`.
- Keep the summary under 72 characters, in the imperative mood.
- A breaking change has `!` after the type and a `BREAKING CHANGE:` footer.
- Example: `fix(axiom): compress back-pressure in one pass`
- The pull request title becomes the commit on `main`, so it must follow the same format.
- `Co-Authored-By` and `Claude-Session` trailers are allowed for AI-assisted work.

## Releases

Each project is released on its own, with a signed annotated tag `<project>-vX.Y.Z`, such as
`axiom-v0.2.0` or `tictactoe-v0.1.0`. The flow:

1. `make release-prepare PROJECT=axiom VERSION=0.2.0` creates `release/axiom-v0.2.0`, moves
   `[Unreleased]` into the new version section, bumps the version files, and commits. It pushes nothing.
2. `make ci`, then `make release-check PROJECT=axiom`.
3. `make release-pr PROJECT=axiom VERSION=0.2.0` pushes the branch and opens the PR.
4. The PR stays open for at least 24 hours, all checks green, then it is squash merged.
5. The owner updates `main` (`git switch main && git pull --ff-only`), then runs
   `make release-tag PROJECT=axiom` and `make release-push PROJECT=axiom`.
6. The release workflow verifies the tag, runs `make ci`, builds the artifacts, waits for approval in the
   `production` environment, and publishes the GitHub release.

Never create a release by hand in the GitHub UI. The workflow creates it. Tags are never moved or deleted.

Anyone, including an agent, can run steps 1 to 3. Steps 5 and 6 are the owner's. Hotfixes use
`make hotfix-start` and a support branch. The full runbook is in
[docs/engineering/release-process.md](docs/engineering/release-process.md). Tag rules are in
[docs/engineering/tag-protection.md](docs/engineering/tag-protection.md).

## Architecture decisions

Record a decision that affects structure, tooling, or a public contract as an ADR in
[docs/engineering/adr/](docs/engineering/adr/). Copy `0000-template.md` to the next number.
