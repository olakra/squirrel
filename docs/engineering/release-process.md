# Release process

This is the runbook for shipping a version of one project. Each project in
`scripts/release/projects.sh` (axiom, tictactoe) releases on its own. The pipeline is
`.github/workflows/release.yml`. The version rules are in [versioning.md](versioning.md). Tags are
never moved or deleted, see [tag-protection.md](tag-protection.md). The branches are described in
[branching-strategy.md](branching-strategy.md).

The owner (@olakra) runs every step that creates a tag or approves a release. An agent or a
contributor may prepare the release pull request, but may not push a tag.

In the commands below, replace `axiom` and `0.2.0` with the project and the version. Run
`make versions` to see each project's version and latest tag.

## Cadence

- Minor releases follow milestones. A milestone ends with a release, and its changelog section
  is the milestone summary.
- Patch releases ship when a fix is ready and the release has no unreleased feature in it. Review
  open fixes at least weekly, and ship a patch when one is waiting.
- There is no release calendar before 1.0.0. The rule is that `main` is always releasable. Any
  release can be cut from `main` at any time.

## Runbook

1. **Decide the version.** Apply the bump rules in [versioning.md](versioning.md) to the entries
   under `## [Unreleased]` in the project's changelog.
2. **Prepare.** From a clean tree:

   ```sh
   make release-prepare PROJECT=axiom VERSION=0.2.0
   ```

   This creates branch `release/axiom-v0.2.0` from `origin/main`. It moves `[Unreleased]` into
   `## [0.2.0] - <today>`, updates the link references, bumps the version files and `Cargo.lock`,
   and makes one signed commit, `chore(axiom): release v0.2.0`. It pushes nothing. Read the diff.
   Any breaking change must be marked **Breaking**.
3. **Check locally.** Run `make ci`, then `make release-check PROJECT=axiom`. `release-check`
   confirms the version files, the tag name, and the changelog section agree. It tags nothing.
4. **Open the release pull request.**

   ```sh
   make release-pr PROJECT=axiom VERSION=0.2.0
   ```

   This pushes the branch and opens the pull request with `gh`. Risk class: normal. Leave it open
   for at least 24 hours (see [change-management.md](change-management.md)). Until it merges, only
   fixes for this release go onto `main` for this project.
5. **Merge.** When every required check is green, squash merge. Auto-merge is allowed. Check that
   the commit on `main` has the subject `chore(axiom): release v0.2.0`.
6. **Tag.** On an up-to-date `main`:

   ```sh
   git switch main && git pull --ff-only
   make release-tag PROJECT=axiom
   ```

   This creates the signed, annotated tag `axiom-v0.2.0` at `HEAD`. Its message is `axiom v0.2.0`
   followed by the changelog section. It pushes nothing. Only an admin can create a release tag,
   see [tag-protection.md](tag-protection.md).
7. **Push the tag.**

   ```sh
   make release-push PROJECT=axiom
   ```

   `release.yml` takes over from here.
8. **Verify.** The `verify` job checks the tag: the tag format, the version files and the changelog
   agree, the tag is annotated, it is reachable from `main` or from `support/axiom-v0.2`, and no
   release exists yet. A failure stops the run before anything is built.
9. **Build.** The `ci` job runs `make ci`. The `build` job runs `make release-artifacts`, which
   builds the project's artifacts and `SHA256SUMS` in `dist/`, then uploads them to the run.
10. **Approve.** The `release` job waits for a reviewer in the `production` environment. Open the
    run, check the build results, and approve. Use this step as the final review of the artifacts.
11. **Publish.** The job checks the checksums, creates the attestation, and creates the GitHub
    release with `gh release create`. The title is `axiom v0.2.0`. The notes come from that
    project's changelog section. The release is marked pre-release if the version has a `-` suffix.
    Only stable axiom releases are marked "Latest". Other projects publish with `--latest=false`.
12. **Post-release check.** Run these from a clean directory:

    ```sh
    gh release download axiom-v0.2.0 --repo olakra/squirrel
    sha256sum -c SHA256SUMS
    gh attestation verify axiom-0.2.0.wasm --repo olakra/squirrel
    ```

    Then confirm on the release page that the notes are right, the pre-release flag is right, and
    the release is marked latest only if it should be. Load the build in the browser once.

## Never create a release in the GitHub UI

The workflow creates the release. Do not use "Draft a new release" or "Create release from tag"
in the GitHub UI, and do not run `gh release create` by hand.

This is what happened to v0.1.2. A release was created by hand in the UI before the workflow
finished. The workflow's `gh release create` then failed, so the published release has no assets:
no wasm and no `SHA256SUMS`. The attestation was created. The fix is an owner action: download the
run's artifacts and upload them to the existing release.

```sh
gh run download 42735109330 -n release-artifacts --repo olakra/squirrel
gh release upload v0.1.2 <files> --repo olakra/squirrel
```

## Hotfix flow

Use this for an emergency (see the risk classes in [change-management.md](change-management.md)).

**If `main` has no unreleased work for the project**, fix on `main` and cut a patch release with the
runbook above. Branch with `make branch TYPE=fix NAME=<short-name>`, add a test that fails before
the fix, and use risk class `emergency`.

**If `main` holds unreleased work for the project**, ship the fix from a support branch:

1. Start the hotfix from the released tag:

   ```sh
   make hotfix-start PROJECT=axiom FROM=0.2.0 NAME=crash-on-load
   ```

   This creates `support/axiom-v0.2` from tag `axiom-v0.2.0` if it is missing, and pushes it. Then it
   creates `hotfix/crash-on-load` from it. Creating the support branch is an owner step.
2. Make the fix, or cherry-pick it from `main` if it already exists there. Add a test that fails
   before the fix. Add a changelog entry under `[Unreleased]`, marked **Fixed**, or **Security** for
   an advisory.
3. Open a pull request into `support/axiom-v0.2` with risk class `emergency`. Squash merge when the
   checks pass.
4. Prepare the patch release on the support branch, and open its pull request into the support
   branch:

   ```sh
   make release-prepare PROJECT=axiom VERSION=0.2.1 BASE=origin/support/axiom-v0.2
   make release-pr PROJECT=axiom VERSION=0.2.1
   ```

5. After the merge, tag on the support branch: `git switch support/axiom-v0.2 && git pull --ff-only`,
   then `make release-tag PROJECT=axiom` and `make release-push PROJECT=axiom`. The cooling-off is
   waived. The approval gate is not.
6. **Forward-port.** Open a normal pull request into `main` with the same fix. Use a `fix(...)`
   subject and add a changelog entry under `[Unreleased]`. Without this step the fix is lost in the
   next minor release.

## Rollback

A published release is never deleted, and a tag is never moved (see
[tag-protection.md](tag-protection.md)). To roll back, use one of these:

1. **Point users to the last good release.** Mark it as latest:

   ```sh
   gh release edit axiom-vX.Y.Z --latest --repo olakra/squirrel
   ```

2. **Warn on the bad release.** Edit its notes to say which version to use instead and why:

   ```sh
   gh release edit axiom-vBAD --notes-file warning.md --repo olakra/squirrel
   ```

3. **Ship the undo.** Revert the faulty commit on `main` through a pull request (see
   [commit-conventions.md](commit-conventions.md) for reverts). Then cut a new patch release that
   contains the revert. This is the fix that stays.

Steps 1 and 2 take minutes and change no history. Step 3 is always needed for a code defect,
because the broken build stays downloadable.

## Records

- Each release has a changelog section, a signed tag, a GitHub release, the artifacts and
  checksums, and a build attestation. Together they identify the exact source and the exact bytes.
- The release run in GitHub Actions shows who approved it. Keep the run link in the release
  pull request.
- The release commit `chore(<project>): release vX.Y.Z` on `main` or on the support branch links the
  version bump to its pull request.
