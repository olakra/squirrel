# Release process

This is the runbook for shipping a version of the axiom crate. The pipeline is
`.github/workflows/release.yml`. The version rules are in [versioning.md](versioning.md). Tags are
never moved or deleted, see [tag-protection.md](tag-protection.md).

The owner (@olakra) runs every step that creates a tag or approves a release. An agent or a
contributor may prepare the release pull request, but may not push a tag.

## Cadence

- Minor releases follow milestones. A milestone ends with a release, and its changelog section
  is the milestone summary.
- Patch releases ship when a fix is ready and the release has no unreleased feature in it. Review
  open fixes at least weekly, and ship a patch when one is waiting.
- There is no release calendar before 1.0.0. The rule is that `main` is always releasable. Any
  release can be cut from `main` at any time.

## Runbook

1. **Freeze.** Open the release pull request. Until it merges, only fixes for this release go onto
   `main`. Merge them through normal pull requests.
2. **Decide the version.** Apply the bump rules in [versioning.md](versioning.md). Write it down in
   the pull request title, for example `chore(axiom): release v0.2.0`.
3. **Changelog.** In the same branch, move the entries under `## [Unreleased]` into a new section
   `## [X.Y.Z] - YYYY-MM-DD`. Put the date of the merge in it. Update the link references at the
   bottom of `projects/axiom/CHANGELOG.md`. Any breaking change must be marked **Breaking**.
4. **Bump the version.** Edit `version` in `projects/axiom/Cargo.toml`. Run `cargo build` in
   `projects/axiom` so `Cargo.lock` picks up the new version. Commit both files in one commit,
   with the type `chore`.
5. **Check locally.** Run `make ci`, then `make release-check`. `release-check` confirms the
   Cargo version, the tag name, and the changelog section agree. It tags nothing.
6. **Open the release pull request.** Use the template. Risk class: normal. Changelog box: ticked.
   Leave it open for at least 24 hours (see [change-management.md](change-management.md)).
7. **Merge.** When every required check is green, squash merge. Check that the commit on `main`
   has the release subject.
8. **Tag.** On an up-to-date `main`, create an annotated tag on the merge commit, then push it:

   ```sh
   git switch main && git pull --ff-only
   git tag -a vX.Y.Z -m "axiom vX.Y.Z"
   git push origin vX.Y.Z
   ```

   The tag ruleset has no bypass actors, so the owner must disable it for the push and enable it
   again straight after. Follow the procedure in [tag-protection.md](tag-protection.md).
9. **Verify.** The `verify` job checks the tag: annotated, equal to the Cargo version, has a
   changelog section, reachable from `origin/main`, and no release exists yet. A failure stops the
   run before anything is built.
10. **Build.** The `ci` job runs `make ci`. The `build` job builds `axiom-X.Y.Z.wasm` and
    `SHA256SUMS`, then uploads them to the run.
11. **Approve.** The `release` job waits for a reviewer in the `production` environment. Open the
    run, check the build results, and approve. Use this step as the final review of the artifacts.
12. **Publish.** The job checks the checksums, extracts the release notes from the changelog,
    creates the attestation, and creates the GitHub release. The release is marked pre-release if
    the version has a `-` suffix.
13. **Post-release check.** Run these from a clean directory:

    ```sh
    gh release download vX.Y.Z --repo olakra/squirrel
    sha256sum -c SHA256SUMS
    gh attestation verify axiom-X.Y.Z.wasm --repo olakra/squirrel
    ```

    Then confirm on the release page that the notes are right, the pre-release flag is right, and
    the release is marked latest only if it should be. Load the wasm in the browser harness once.

## Hotfix flow

Use this for an emergency (see the risk classes in [change-management.md](change-management.md)).

Because `main` is always releasable, a hotfix is a fix on `main` followed by a patch release. The
tag rule requires every release tag to sit on `main`, so a hotfix cannot ship from a branch that
`main` does not contain.

1. Branch from the released tag, so the fix contains nothing else:

   ```sh
   git switch -c fix/<short-name> vX.Y.Z
   ```

2. Make the fix, or cherry-pick it from `main` if it already exists there. Add a test that fails
   before the fix.
3. Add a changelog entry under `[Unreleased]`. Mark it **Fixed**, or **Security** for an advisory.
4. Open a pull request into `main` with risk class `emergency`. Squash merge when the checks pass.
5. Cut a patch release from `main` with the steps above. The version is `X.Y.(Z+1)`.

If `main` holds unreleased work that must not ship yet, do not work around the tag rule. Revert
that work on `main` through a pull request first, then cut the hotfix.

## Rollback

A published release is never deleted, and a tag is never moved (see
[tag-protection.md](tag-protection.md)). To roll back, use one of these:

1. **Point users to the last good release.** Mark it as latest:

   ```sh
   gh release edit vX.Y.Z --latest --repo olakra/squirrel
   ```

2. **Warn on the bad release.** Edit its notes to say which version to use instead and why:

   ```sh
   gh release edit vBAD --notes-file warning.md --repo olakra/squirrel
   ```

3. **Ship the undo.** Revert the faulty commit on `main` through a pull request (see
   [commit-conventions.md](commit-conventions.md) for reverts). Then cut a new patch release that
   contains the revert. This is the fix that stays.

Steps 1 and 2 take minutes and change no history. Step 3 is always needed for a code defect,
because the broken build stays downloadable.

## Records

- Each release has a changelog section, a tag, a GitHub release, the wasm and checksums, and a
  build attestation. Together they identify the exact source and the exact bytes.
- The release run in GitHub Actions shows who approved it. Keep the run link in the release
  pull request.
