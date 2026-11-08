# Tag protection and immutable releases

A release tag is a permanent pointer to a commit. Users, the attestation, and the checksums all
depend on it. So a `v*` tag is never moved, re-pointed, deleted, or pushed twice. This page explains
how the repository enforces that, and what the owner must set in the UI.

## Rules

1. **Never move a tag.** Do not force-push a tag. Do not delete and recreate one. If a tag points at
   the wrong commit, leave it, cut a new version, and follow [release-process.md](release-process.md).
2. **Never delete a tag or a release.** Published releases stay. Mark a bad one with a warning
   instead (see the rollback section of the release process).
3. **Create a tag once, for one version.** The `verify` job refuses a tag whose release already
   exists, and it refuses a tag that is not annotated or not on `main`.

## Ruleset: `.github/rulesets/tags.json`

The ruleset targets `refs/tags/v*` and does three things:

| Rule | Effect |
|---|---|
| `creation` | Only the process that the ruleset allows can create a release tag. |
| `deletion` | No one can delete a release tag. |
| `non_fast_forward` | No one can move a tag to a commit that is not a descendant of the old one. |

`bypass_actors` is empty, so the rules bind everyone, including the owner and repository admins.

The effect for the owner: with the ruleset active, a new `v*` tag cannot be pushed. To cut a release,
the owner does this, in order:

1. Disable the `release tags` ruleset in Settings, then Rules, then Rulesets. Use "Disabled", not
   "Evaluate". "Evaluate" logs the attempt but still lets it through.
2. Push the annotated tag (see the release runbook, step 8).
3. Set the ruleset back to "Active" straight away. Check that the tag now shows as protected.

This keeps tag creation a deliberate, recorded act. A decision for the owner: if the disable step
is too much friction, add the owner as a bypass actor for the `creation` rule only. That removes
this procedure at the cost of letting the owner move a tag. Do not add a bypass for `deletion` or
`non_fast_forward`.

## Applying the ruleset

`scripts/ci/apply-branch-protection.sh --dry-run` prints the payload of `tags.json` and of `main.json`
without calling the API. `--apply` creates or updates both by name. Run it once, as the owner, and
read [branch-protection.md](branch-protection.md) first.

Do not use `--apply` to re-enable the tag ruleset during a release. Use the UI, so the change shows
up in the settings log at the moment it happens.

## Immutable releases (owner, in the UI)

GitHub has a repository setting for immutable releases. When it is on, GitHub refuses to change a
published release's assets or to re-point its tag. The rulesets do not turn it on, so the owner must:

1. Open Settings, then General, then find "Releases" (labelled immutable releases in current
   GitHub versions).
2. Turn on immutable releases.
3. Confirm with a test release on a scratch repository, or by reading the GitHub documentation for
   the current wording, because the label has changed in the past.

Immutable releases protect the assets and the tag after publication. The tag ruleset protects the
tag before publication, when a release does not yet exist.

## What is checked where

| Check | Where it runs | What it blocks |
|---|---|---|
| Tag ruleset | GitHub, on every push of a `v*` tag | Creating (by unauthorised people), deleting, or moving a tag |
| `verify` job, release-exists check | `.github/workflows/release.yml` | Publishing again to a tag that already has a release, including drafts |
| `verify` job, annotated and on `main` | `.github/workflows/release.yml` | Lightweight tags, and tags off `main` |
| `production` environment reviewer | `.github/workflows/release.yml`, job `release` | Publishing without an approval |
| Immutable releases | GitHub, after publication | Changing assets or re-pointing a published release's tag |
