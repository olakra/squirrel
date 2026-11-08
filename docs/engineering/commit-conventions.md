# Commit conventions

Commits follow [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/). The
type says what kind of change the commit makes. The changelog and the version bump depend on it.

## Format

```text
<type>(<scope>)!: <summary>

<body, optional>

<footers, optional>
```

- **type**: one word from the table below. Lowercase.
- **scope**: optional. The area that changed, in lowercase: `axiom`, `sim`, `wasm`, `ci`, `docs`,
  `release`. It may contain `.`, `_`, `/`, or `-`.
- **!**: optional. Marks a breaking change. See the footer rule below.
- **summary**: imperative mood ("add", not "added"). Starts after `: `. No full stop at the end.
- The whole subject line is 72 characters or fewer.

### Types

| Type | Use it for | Version effect (pre-1.0) |
|---|---|---|
| `feat` | A new behaviour or public item | minor |
| `fix` | A bug fix | patch |
| `perf` | A speed or memory change that keeps every output the same | patch |
| `refactor` | A code change with no behaviour change | none |
| `test` | Tests only | none |
| `docs` | Documentation only | none |
| `build` | Build scripts, toolchain, or dependency changes | none, unless it changes the artifact |
| `ci` | Workflows, CI scripts, and gates | none |
| `chore` | Maintenance, such as the release commit | none |
| `revert` | Undoes an earlier commit | the reverted type's effect, decided at release |

Two more rules apply to every type:

- A commit that changes a golden hash is a `feat` or `fix` with a breaking footer, so its
  version effect is visible. See [versioning.md](versioning.md).
- Use `refactor` for a change that moves code. Do not hide a behaviour change inside it.

### Examples

```text
feat(axiom): add the inserter state machine

Inserters now reserve their source before picking up an item, so two inserters cannot take
the same item in one tick.
```

```text
fix(sim): compress back-pressure in one pass

Two passes produced a different state hash on Windows. One pass removes the platform
difference.
```

```text
feat(axiom)!: change axiom_feed to return a status code

BREAKING CHANGE: axiom_feed returned 0 on failure. It now returns a status code. The browser
harness must check for the new codes.
```

## Breaking changes

A breaking change needs both of these:

1. A `!` after the type or scope, so the subject line shows it.
2. A footer that starts with `BREAKING CHANGE:` (both words in capitals, a colon, a space), then
   a description of what breaks and what to do instead.

The commit-lint check accepts the `!` form. The footer is not checked by a tool. The reviewer
checks it.

## Footers and trailers

Footers come after the body, separated by one blank line. Allowed footers:

- `BREAKING CHANGE: <description>`
- `Refs: #<issue>` or `Closes: #<issue>`
- `Co-Authored-By: <name> <email>`. This is allowed for AI-assisted work. The repository history
  uses it, with the `Claude-Session:` trailer that records the session. Keep both.
- `Reviewed-by:` or `Signed-off-by:`, when a reviewer or the author wants one.

The body explains why the change was made. It wraps at 72 characters. It does not repeat the diff.

## Reverts

Use `git revert <sha>`. Git writes the subject as `Revert "<original subject>"`. The commit-lint
check accepts that form. The body should state the reason and the reverted SHA.

If you write the revert by hand, use the `revert` type:

```text
revert: undo the chunk sleep change

This reverts commit 1a2b3c4. Waking chunks on every write broke the determinism test.
```

## Squash merges and the PR title

Pull requests are merged by squash only (see [change-management.md](change-management.md)). The
squash commit on `main` takes its subject from the pull request title. So the title must be a
valid subject too. The `commit-lint` job checks the title as well as every commit in the pull
request. The repository setting "Default to pull request title" must be on. See
[branch-protection.md](branch-protection.md).

## What the checks enforce

| Check | Where | Blocks |
|---|---|---|
| `commit-lint` (commits and PR title) | `.github/workflows/ci.yml`, job `commit-lint`, using `scripts/ci/check-commit-messages.sh` | Merge |
| Local run | `make commit-lint` (compares against `origin/main`) | Nothing. It runs before you push. |
