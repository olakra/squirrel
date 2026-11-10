# Branch protection and environments

These are the repository settings that enforce [change-management.md](change-management.md),
[branching-strategy.md](branching-strategy.md), and [release-process.md](release-process.md). The same
rules exist in two forms: the ruleset files in `.github/rulesets/`, and the steps below for the UI.
Use one route, then check the other against it.

The owner (@olakra) sets these. An agent cannot.

## Why this page changed (2026-11-09)

A check of the live settings found that the rulesets in this repository had never been applied:

- `main` had classic branch protection with zero required status checks, and admins were not
  enforced. The repository allowed merge commits and rebase merges. That is how the merge commits in
  #29 and #30, and the non-conventional subject `release/0.1.1 (#39)`, reached `main`.
- The live tag ruleset was named `tags` and had an empty include list, so it protected nothing.
- The `production` environment had no reviewer and no tag policy.

The script now applies the rulesets, the merge settings, and the environment in one run.

## Option A: apply everything with the script

Requires `gh`, logged in as an admin of the repository.

```sh
make github-protect-dry   # print every payload, change nothing
make github-protect       # apply the rulesets, the repository merge settings, and the environment
```

Both run `scripts/ci/apply-branch-protection.sh`. It applies:

- The branch rulesets `main` (`.github/rulesets/main.json`) and `support`
  (`.github/rulesets/support.json`).
- The tag rulesets `release tags` (`tags.json`) and `release tag creation` (`tag-creation.json`). See
  [tag-protection.md](tag-protection.md).
- The repository merge settings in step 3 below.
- The `production` environment in step 4 below.

The script matches rulesets by name, so a second run updates them and does not duplicate them.

### Remove the classic branch protection

The script does not delete the old classic branch protection on `main`. Once the `main` ruleset is
active, the owner removes it, in Settings, then Branches, or with:

```sh
gh api -X DELETE repos/olakra/squirrel/branches/main/protection
```

Until it is removed, both apply, and the classic rule's weaker settings confuse the merge box.

## Option B: the same settings in the UI

### 1. Branch rulesets "main" and "support"

Settings, then Rules, then Rulesets, then New branch ruleset. Create two with the same rules:

- `main`: target the default branch.
- `support`: target `refs/heads/support/**`.

For both:

- Enforcement: Active.
- Bypass list: empty. This enforces the rules for admins too.
- Restrict deletions: on.
- Block force pushes: on.
- Require linear history: on.
- Require signed commits: on. GitHub signs squash merges made in the UI or by auto-merge, so those
  always pass.
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
| `tictactoe` | ci.yml, job `tictactoe` |
| `cargo-deny` | ci.yml, job `supply-chain` |
| `markdownlint` | ci.yml, job `docs` |
| `scripts-test` | ci.yml, job `scripts-test` |
| `commit-lint` | ci.yml, job `commit-lint` |
| `changelog-check` | ci.yml, job `changelog-check` |
| `branch-name` | ci.yml, job `branch-name` |

Check names appear in the list only after each job has run at least once. If a name is missing,
open a pull request first so the check runs, then add it.

- Merge queue: leave off. With one maintainer and squash-only merges, a queue adds waiting and no
  safety. See the switch list in [branching-strategy.md](branching-strategy.md#when-the-team-grows).

### 2. Tag rulesets

Create `release tags` and `release tag creation` as described in
[tag-protection.md](tag-protection.md).

### 3. Repository settings

Settings, then General, then Pull Requests:

- Allow squash merging: on. Allow merge commits: off. Allow rebase merging: off.
- Default commit message for squash merging: "Pull request title and description". The commit
  subject on `main` then equals the title, which `commit-lint` checks.
- Allow auto-merge: on.
- Automatically delete head branches: on.

Settings, then General, then find "Releases", and turn on immutable releases (see
[tag-protection.md](tag-protection.md)).

### 4. Environment "production"

Settings, then Environments, then New environment, named `production`.

- Required reviewers: @olakra.
- Prevent self-review: **off**. With one owner, who both pushes the tag and approves, on would block
  every release. Turn it on when a second maintainer can approve.
- Deployment branches and tags: Selected branches and tags, with the tag pattern `*-v*.*.*`. Only
  project release tags can use this environment. The old pattern `v*.*.*` is no longer needed.

### 5. Labels

Create these once, so the labels used by the templates and `.github/release.yml` exist:

- `no-changelog`: the changelog check is skipped. The pull request body must give the reason.
- `breaking-change`, `feat`, `fix`, `perf`, `refactor`, `docs`, `test`, `build`, `ci`, `chore`,
  `revert`: the categories in the generated release notes.

## Required checks for a merge, and for a release

| Blocks a merge to `main` or `support/**` | Blocks a release |
|---|---|
| Every check in the table in step 1 | `verify` (tag rules, sources agree, annotated, on `main` or the support branch, no existing release) |
| The ruleset rules (linear history, squash only, up to date, signed) | `ci` (`make ci`, the same gates) |
| | `build` (`make release-artifacts`, checksums) |
| | `release` (environment approval, checksum check, attestation) |

## Checking the result

- Open a pull request with a bad commit subject. `commit-lint` must fail, and the merge button must
  stay disabled.
- Open a pull request from a branch with a bad name, such as `My_Branch`. `branch-name` must fail.
- Try a direct push to `main`. It must be refused.
- As a non-admin, try to push a tag such as `axiom-v9.9.9`. It must be refused.
- Run `make scripts-test` and `make release-check PROJECT=axiom` locally. Both must pass before the
  rules are considered live.
