# Change management

Every change to this repository moves through the same stages: propose, implement, review, merge,
release, and, after an incident, review. Each stage leaves a record, so any change can be traced
from the code on `main` back to the reason for it.

## Stages

1. **Propose.** Open an issue for a bug or a feature. For an architectural change, write an ADR
   in `docs/engineering/adr/` first, and merge it before the code. An ADR changes structure,
   tooling, or a public contract.
2. **Implement.** Branch from `main`. Use one of these names:
   - `feature/<short-name>` for new behaviour
   - `fix/<short-name>` for a bug fix
   - `chore/<short-name>` for maintenance, docs, or pipeline work
   Keep the pull request under about 400 changed lines. Split it if it is larger.
3. **Review.** Fill in the pull request template: change type, risk class, rollback plan,
   determinism impact, changelog entry or label reason, and test evidence. Review against
   [code-review.md](code-review.md).
4. **Merge.** Squash merge only. The history stays linear, and each pull request becomes one commit
   on `main`. The commit subject is the pull request title, so the title follows
   [commit-conventions.md](commit-conventions.md).
5. **Release.** Changes on `main` reach users only through a release. See
   [release-process.md](release-process.md).
6. **Post-incident review.** After any incident, or any emergency change, write a short review
   (see below).

## Review and the single-owner exception

[CODEOWNERS](../../.github/CODEOWNERS) names one owner, and GitHub does not count an author's own
approval. So a rule that requires two approvals, or even one, would block every change. The
branch ruleset therefore requires zero approving reviews. The pull request still must be merged
through the ruleset, and every required check must pass. The reviewer is the author, using the
checklist.

The compensating controls are:

1. **Required checks.** Every gate in [quality-gates.md](quality-gates.md) is a required check, plus
   `commit-lint` and `changelog-check`. A red job blocks the merge. Nothing merges with a skipped
   required check.
2. **Self-review checklist.** The author works through [code-review.md](code-review.md) in order,
   and ticks the self-review box in the pull request. Blocking review points are answered in the
   pull request before merge.
3. **24-hour cooling-off for release commits.** The release pull request (version bump and
   changelog) stays open for at least 24 hours before its tag is pushed. This gives time to spot
   a mistake. It does not apply to an emergency change.
4. **Protected release environment.** The `production` environment needs an approval before any
   release is published. See [branch-protection.md](branch-protection.md).
5. **Immutable tags.** A tag cannot be moved or deleted, so a published release is a fixed point.
   See [tag-protection.md](tag-protection.md).
6. **Enforced for admins.** The ruleset has no bypass actors. Nobody, including the owner, merges
   around a failed check.

When a second maintainer joins, set `required_approving_review_count` to 1 and
`require_code_owner_review` to true in `.github/rulesets/main.json`, then re-apply it. Remove this
exception at that point.

## Risk classes

Each pull request declares one class in its template. The class sets the minimum steps. A reviewer
may ask for more, never less.

| Class | Typical changes | Required before merge | Required before release | Release timing |
|---|---|---|---|---|
| **Standard** | Docs, tests with no runtime effect, `chore`, pipeline changes that do not change a gate | All required checks green. Template complete. Self-review ticked. | Normal release flow | Any time the owner chooses |
| **Normal** | Any change to the crate's behaviour, dependencies, build flags, or public surface. Every `feat` and `fix`. | All of the above, plus: changelog entry (or label with reason), determinism impact stated, rollback plan written, ADR if architectural, benchmark delta for hot paths | Normal release flow, including the 24-hour cooling-off for the release commit | Scheduled release |
| **Emergency** | A fix for a released version that users need now (a crash, a broken export, a determinism break in a shipped build) | All required checks still green. No direct push. A PR is still required. Template complete, with the rollback plan first. | Hotfix flow (see [release-process.md](release-process.md)). Cooling-off is waived, the approval gate is not. | Immediately, then a post-incident review |

Emergency changes do not skip a check. If a check is wrong, fix the check in its own pull request,
and merge that first.

## Post-incident review

Write one for every emergency change and for any released defect that reached users. Keep it short
and blameless. Put it in `docs/engineering/incidents/` through a pull request (class `standard`),
named `YYYY-MM-DD-<short-name>.md`. Include:

1. **Summary.** What broke, which version, and for how long.
2. **Impact.** Who or what was affected. Which releases.
3. **Timeline.** Key times, in UTC.
4. **Root cause.** The first thing that was wrong, and why no gate caught it.
5. **What went well.**
6. **Actions.** Each action has an owner and a pull request or issue. An action that would add a
   gate names the gate.

Write the review within five working days of the fix.
