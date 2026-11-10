# Tag protection and immutable releases

A release tag is a permanent pointer to a commit. Users, the attestation, and the checksums all
depend on it. So a release tag is never moved, re-pointed, deleted, or pushed twice. This page
explains how the repository enforces that, and what the owner must set in the UI.

Release tags are `<project>-vX.Y.Z`, plus the legacy `v0.1.2`. See [versioning.md](versioning.md).

## Rules

1. **Never move a tag.** Do not force-push a tag. Do not delete and recreate one. If a tag points at
   the wrong commit, leave it, cut a new version, and follow [release-process.md](release-process.md).
2. **Never delete a tag or a release.** Published releases stay. Mark a bad one with a warning
   instead (see the rollback section of the release process).
3. **Create a tag once, for one version.** The `verify` job refuses a tag whose release already
   exists. It refuses a tag that is not annotated, or not on `main` or the project's support branch.
4. **Sign every tag.** `make release-tag` creates an annotated, signed tag with `git tag -s`. The
   signing setup is in [claude-code-setup.md](claude-code-setup.md#signed-commits).

## Rulesets

Two tag rulesets cover the same patterns:

- `refs/tags/v*`: the legacy tag `v0.1.2`, and any other unprefixed tag.
- `refs/tags/*-v*`: every project tag, such as `axiom-v0.2.0`.

| Ruleset | File | Rules | Bypass |
|---|---|---|---|
| `release tags` | `.github/rulesets/tags.json` | `deletion`, `non_fast_forward`, `update` | None. The rules bind everyone, admins included. |
| `release tag creation` | `.github/rulesets/tag-creation.json` | `creation` | Repository admin role, always. |

The effect:

- Only a repository admin can create a release tag. A contributor, an agent, or a bot cannot.
- Nobody, including an admin, can delete a release tag or move it.

This replaces the old procedure, where the owner disabled the tag ruleset, pushed the tag, and enabled
it again. No ruleset is disabled during a release. The admin bypass covers creation only. Do not add
a bypass to `release tags`.

## Applying the rulesets

`make github-protect-dry` prints the payload of every ruleset, the repository merge settings, and the
`production` environment, without calling the API. `make github-protect` creates or updates them by
name. Both run `scripts/ci/apply-branch-protection.sh`. Run them as the owner, and read
[branch-protection.md](branch-protection.md) first.

## Immutable releases (owner, in the UI)

GitHub has a repository setting for immutable releases. When it is on, GitHub refuses to change a
published release's assets or to re-point its tag. The rulesets do not turn it on, so the owner must:

1. Open Settings, then General, then find "Releases" (labelled immutable releases in current
   GitHub versions).
2. Turn on immutable releases.
3. Confirm with a test release on a scratch repository, or by reading the GitHub documentation for
   the current wording, because the label has changed in the past.

Immutable releases protect the assets and the tag after publication. The tag rulesets protect the
tag before publication, when a release does not yet exist.

An immutable release also cannot get its assets later. So a release created by hand in the UI, before
the workflow uploads its assets, stays empty. That is one more reason never to create a release in
the UI, see [release-process.md](release-process.md#never-create-a-release-in-the-github-ui).

## What is checked where

| Check | Where it runs | What it blocks |
|---|---|---|
| `release tag creation` ruleset | GitHub, on every push of a matching tag | Creating a release tag without the admin role |
| `release tags` ruleset | GitHub, on every push of a matching tag | Deleting or moving a release tag, for everyone |
| `make release-check PROJECT=<project>` | Local, before the release pull request | Version files, tag name, and changelog section that disagree |
| `verify` job, tag format and sources agree | `.github/workflows/release.yml` | A tag that does not match the project's version and changelog |
| `verify` job, annotated and on the right branch | `.github/workflows/release.yml` | Lightweight tags, and tags off `main` or `support/<project>-vX.Y` |
| `verify` job, release-exists check | `.github/workflows/release.yml` | Publishing again to a tag that already has a release, including drafts |
| `production` environment | `.github/workflows/release.yml`, job `release` | Publishing without an approval, or from a ref that is not a `*-v*.*.*` tag |
| Immutable releases | GitHub, after publication | Changing assets or re-pointing a published release's tag |
