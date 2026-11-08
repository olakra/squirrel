<!-- markdownlint-disable-file MD041 -->
## What and why

<!-- One or two sentences. Link the issue or ADR if there is one. -->

## Change type

Use the type of the PR title (see docs/engineering/commit-conventions.md).

- [ ] `feat`: new behaviour
- [ ] `fix`: bug fix
- [ ] `perf`: performance change
- [ ] `refactor`: no behaviour change
- [ ] `test`: tests only
- [ ] `docs`: documentation only
- [ ] `build`, `ci`, or `chore`: build, pipeline, or maintenance
- [ ] `revert`: undoes an earlier change
- [ ] Breaking change: the title has `!` and the body has a `BREAKING CHANGE:` footer

## Risk class

Choose one (see docs/engineering/change-management.md).

- [ ] standard: docs, tests, or maintenance with no runtime effect
- [ ] normal: any change to the crate's behaviour, dependencies, or build
- [ ] emergency: a fix for a released version that cannot wait for the normal flow

## Rollback plan

<!-- How to undo this change. For example: "revert this commit; no data or format changed",
     or "turn off feature X". State it before merge. -->

## Determinism impact

<!-- Does this change the fixed-tick state hash, the save format, or the WASM export list?
     Write "none", or describe the change and which golden test values were updated. -->

## CHANGELOG

- [ ] `projects/axiom/CHANGELOG.md` has an entry under `[Unreleased]`
- [ ] Or the `no-changelog` label is set. Reason:

<!-- Required when the label is set and projects/axiom/src or Cargo.toml changed. -->

## Test evidence

<!-- Paste the result of `make ci`, or the failing job and why it is expected. For a hot-path
     change, add the benchmark before and after on the same machine. -->

## Checklist

- [ ] Tests added or updated for every behaviour change
- [ ] Determinism: simulation changes have a determinism test (fixed-tick state hash), or "N/A"
- [ ] Benchmarks: hot-path changes include a benchmark delta, or "N/A"
- [ ] Docs updated (ARCHITECTURE.md, README, or docs/engineering) where behaviour or structure changed
- [ ] ADR added if this is an architectural decision (see docs/engineering/adr/)
- [ ] `make ci` passes locally
- [ ] Self-review done against docs/engineering/code-review.md
- [ ] Changed lines are roughly 400 or fewer, or the PR says why it cannot be split

## Notes for reviewers

<!-- Anything a reviewer should look at first. Mention any benchmark regression and why it is acceptable. -->
