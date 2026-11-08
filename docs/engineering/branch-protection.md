# Branch protection and environments

These are the repository settings that enforce [change-management.md](change-management.md) and
[release-process.md](release-process.md). The same rules exist in two forms: the ruleset files in
`.github/rulesets/`, and the steps below for the UI. Use one route, then check the other against it.

The owner (@olakra) sets these. An agent cannot.

## Option A: apply the rulesets with the script

Requires `gh`, logged in as an admin of the repository.

```sh
scripts/ci/apply-branch-protection.sh --dry-run   # print both payloads, change nothing
scripts/ci/apply-branch-protection.sh --apply     # create or update the "main" and "release tags" rulesets
```

The script matches rulesets by name, so a second run updates them and does not duplicate them. It
does not set the repository settings in the "Pull requests" section or the `production`
environment. Do those in the UI (steps 4 and 5 below).

## Option B: the same settings in the UI

### 1. Branch ruleset "main"

Settings, then Rules, then Rulesets, then New branch ruleset.

- Name: `main`. Enforcement: Active. Target: default branch.
- Bypass list: empty. This enforces the rules for admins too.
- Restrict deletions: on.
- Block force pushes: on.
- Require linear history: on.
- Require a pull request before merging:
  - Required approvals: 0. This is the single-owner exception (see
    [change-management.md](change-management.md)). Raise it to 1 when a second maintainer joins.
  - Dismiss stale pull request approvals when new commits are pushed: on.
  - Require conversation resolution before merging: on.
  - Allowed merge methods: Squash only.
- Require status checks to pass:
  - Require branches to be up to date before merging: on.
  - Add these checks, by exact name:

| Check name | Source job |
|---|---|
| `fmt` | ci.yml, job `fmt` |
| `clippy` | ci.yml, job `clippy` |
| `test (ubuntu-latest)` | ci.yml, job `test`, matrix |
| `test (macos-latest)` | ci.yml, job `test`, matrix |
| `test (windows-latest)` | ci.yml, job `test`, matrix |
| `coverage gate` | ci.yml, job `coverage` |
| `bench smoke` | ci.yml, job `bench-smoke` |
| `wasm build` | ci.yml, job `wasm` |
| `cargo-deny` | ci.yml, job `supply-chain` |
| `markdownlint` | ci.yml, job `docs` |
| `commit-lint` | ci.yml, job `commit-lint` |
| `changelog-check` | ci.yml, job `changelog-check` |

Check names appear in the list only after each job has run at least once. If a name is missing,
open a pull request first so the check runs, then add it.

- Merge queue: leave off. With one maintainer and squash-only merges, a queue adds waiting and no
  safety. Turn it on when several people merge in the same hour.

### 2. Tag ruleset "release tags"

Create it as described in [tag-protection.md](tag-protection.md): target tag, pattern `v*`, rules
creation, deletion, and non-fast-forward, with no bypass actors.

### 3. Repository settings

Settings, then General, then Pull Requests:

- Allow squash merging: on. Allow merge commits: off. Allow rebase merging: off.
- Default to pull request title for squash merge commit messages: on. The commit subject on `main`
  then equals the title, which `commit-lint` checks.
- Automatically delete head branches: on.

Settings, then General, then find "Releases", and turn on immutable releases (see
[tag-protection.md](tag-protection.md)).

### 4. Environment "production"

Settings, then Environments, then New environment, named `production`.

- Required reviewers: @olakra.
- Prevent self-review: **off**. With one owner, who both pushes the tag and approves, on would block
  every release. Turn it on when a second maintainer can approve.
- Deployment branches and tags: Selected branches and tags, with the tag pattern `v*.*.*`. Only
  release tags can use this environment.

### 5. Labels

Create these once, so the labels used by the templates and `.github/release.yml` exist:

- `no-changelog`: the changelog check is skipped. The pull request body must give the reason.
- `breaking-change`, `feat`, `fix`, `perf`, `refactor`, `docs`, `test`, `build`, `ci`, `chore`,
  `revert`: the categories in the generated release notes.

## Required checks for a merge, and for a release

| Blocks a merge to `main` | Blocks a release |
|---|---|
| Every check in the table in step 1 | `verify` (tag rules, changelog section, reachable from `main`, no existing release) |
| The ruleset rules (linear history, squash only, up to date) | `ci` (`make ci`, the same gates) |
| | `build` (wasm, checksums) |
| | `release` (environment approval, checksum check, attestation) |

## Checking the result

- As a non-owner, or with the owner's own account in a test branch, open a pull request with a bad
  commit subject. `commit-lint` must fail, and the merge button must stay disabled.
- Try a direct push to `main`. It must be refused.
- Run `make scripts-test` and `make release-check` locally. Both must pass before the rules are
  considered live.
