# ADR 0006: Tag releases per project and branch from a single trunk

- **Status:** Accepted
- **Date:** 2026-11-09
- **Deciders:** owner

## Context

- The repository holds two projects that ship: axiom and tic-tac-toe. Each has its own version, its
  own changelog, and its own users.
- Release tags were `vX.Y.Z`. A `v` tag names the whole repository, so it cannot say which project it
  releases. [versioning.md](../versioning.md) already said to move to a project prefix before a second
  project tags anything. Tic-tac-toe is about to tag its first release.
- The first axiom release is tagged `v0.1.2`. Tags are never moved or deleted, see
  [tag-protection.md](../tag-protection.md).
- A check of the GitHub settings on 2026-11-09 found gaps. `main` had classic branch protection with
  no required checks and no admin enforcement, and merge commits and rebase merges were allowed. The
  tag ruleset had an empty include list, so it protected nothing. The `production` environment had no
  reviewer and no tag policy.
- The v0.1.2 release was created by hand in the UI before the workflow ran. The workflow's
  `gh release create` then failed, and the release has no assets.
- The old tag procedure asked the owner to disable the tag ruleset, push the tag, and enable it again.
  That is easy to get wrong and leaves a window with no protection.
- The repository has one owner, who works with Claude. Every merge is a squash merge, and `main` is
  always releasable.

Alternatives considered:

1. **Keep repository-wide tags (`vX.Y.Z`).** Rejected. Two projects with independent versions would
   collide on the same tag names, and a tag would not say what it releases.
2. **Classic git-flow with a `develop` branch.** Rejected. With squash-only merges, a `develop` to
   `main` promotion either squashes many pull requests into one commit, which loses history and breaks
   the per-PR changelog and `commit-lint`, or needs merge commits, which break linear history. A second
   integration branch doubles CI runs and merge conflicts. `main` is already releasable.
3. **Release automation with release-please or changesets.** Rejected for now. The release scripts
   are plain shell and work offline, and the owner wants the version bump made by hand in a reviewed
   pull request. Revisit when the team grows.

## Decision

We will tag each release as `<project>-vMAJOR.MINOR.PATCH[-prerelease]`, for example `axiom-v0.2.0`.
The projects, their version files, and their changelogs are listed in one registry,
`scripts/release/projects.sh`. The legacy tag `v0.1.2` stays as axiom's first release.

We will develop on a single trunk, `main`, with short-lived work branches, release prep branches
(`release/<project>-vX.Y.Z`), and support branches (`support/<project>-vX.Y`) for hotfixes to a
released line. See [branching-strategy.md](../branching-strategy.md).

We will let only repository admins create release tags, through a bypass on a tag creation ruleset.
A second tag ruleset blocks deletion and moves for everyone. Tags are annotated and signed.

We will require signed commits on `main` and on `support/**`.

## Consequences

- Each project releases on its own schedule. A tag says which project and which version it is.
- The owner creates a tag with `make release-tag` and pushes it with `make release-push`. No ruleset
  is disabled during a release.
- The tag pattern, the release workflow, the release check, and the `production` tag policy all
  change to the prefixed form. Scripts read the registry, so a third project is one registry entry.
- The legacy pattern `v*` stays protected, so `v0.1.2` can never be moved.
- Only one release per repository can be marked "Latest" on GitHub. Stable axiom releases take it.
  Other projects publish with `--latest=false`.
- Hotfixes to an older line need a support branch and a forward-port to `main`. That is extra work,
  and it is only needed when `main` holds unreleased work for the project.
- Every contributor needs a signing key registered on GitHub to push signed commits. See
  [claude-code-setup.md](../claude-code-setup.md).
- Follow-up: when a second maintainer joins, raise the review counts and revisit release automation.
  The switch list is in [branching-strategy.md](../branching-strategy.md).
