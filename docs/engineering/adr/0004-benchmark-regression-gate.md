# ADR 0004: Keep the benchmark regression rule as a review rule, not a required check

- **Status:** Accepted
- **Date:** 2026-11-08
- **Deciders:** repository owner (olakra), through the owner's delegation for non-owner tasks

## Context

The spec asks that the stress scenario (100,000 items on 10,000 belt segments) hold 60 ticks per
second. [quality-gates.md](../quality-gates.md) states a regression rule: a change that makes the
stress scenario more than 10% slower fails review until the pull request explains why. No automated
comparison exists. The CI bench job runs `--quick` and checks only that result lines are printed.

Facts that bear on the choice:

- Timing noise is large. [benches/README.md](../../../projects/axiom/benches/README.md) records that
  `rear_push` varied by about 25% between back-to-back runs on a shared machine, with no code change.
- Quick and full results are not comparable. CI runs quick. The stress scenario is defined at full size.
- No reference machine is named. Issue #3 asks for one, and it is not yet recorded.
- No benchmark store keyed by commit exists.
- [quality-gates.md](../quality-gates.md) already says shared CI runners vary too much from run to
  run to fail a build on a timing difference.

Options considered:

1. **Hard CI threshold.** Fail the build when the stress scenario is more than 10% slower than a
   stored baseline. Rejected for now. A 10% limit is smaller than the 25% noise measured on a shared
   machine, so the check would fail on noise. The likely result is that people rerun until it passes.
2. **Self-hosted benchmark runner with fixed hardware.** Not chosen now. It needs hardware, an
   isolated runner, and a benchmark store. None of these exists. This option is the route to a
   required check later.
3. **Cloud benchmarking service.** Not chosen. It adds an external service and its data handling.
   No service has been evaluated.
4. **Review rule with a tracked trend.** Chosen. A reviewer checks the stress scenario with
   before-and-after runs on the same machine.

## Decision

We will treat benchmarks as a tracked trend and a review rule, not as a required CI check. A change
that makes the stress scenario more than 10% slower (full size, `ns_per_tick`, same machine) must
justify the cost in its pull request. CI keeps the bench smoke job, which checks only that the suite
runs.

This decision falls within the owner's delegation for non-owner tasks. It is Accepted on that basis,
without a separate owner sign-off. The owner can replace it with a later ADR at any time.

## How a reviewer applies the rule

1. Ask the author for before and after results for any hot-path change. Both runs use the same
   machine and the same mode (full against full).
2. Check the state hash first. A changed `state_hash` means the simulated state changed. That is a
   behaviour change, and [versioning.md](../versioning.md) governs it. It is not only a speed question.
3. Run each side three or more times and compare medians, as
   [benches/README.md](../../../projects/axiom/benches/README.md) describes.
4. Divide the after median by the before median for `stress_100k_items_10k_segments.ns_per_tick`.
   A ratio above 1.10 needs a written justification in the pull request. A ratio below 1.00 is faster.

   ```sh
   grep '^bench stress_100k_items_10k_segments.ns_per_tick' before.txt after.txt
   ```

5. If the difference is within the run-to-run spread the author shows, the rule does not apply. Say
   so in the review.
6. Record the numbers in the pull request description.

## Consequences

- Easier: no red builds caused by runner noise, and no hardware to buy or maintain.
- Harder: the rule depends on the reviewer running the comparison. A slow change can merge if the
  reviewer does not check. The hot-path requirement in [quality-gates.md](../quality-gates.md) is the
  control for this.
- Ruled out for now: a required timing check on shared CI runners.
- Follow-up work: name the reference machine and record the target baseline. See
  [performance-budget.md](../performance-budget.md) and issue #3. This decision creates no follow-up
  issue for a benchmark store.

## What would change the decision

- A named reference machine whose repeated runs agree closely enough that a 10% change is not noise.
  The owner sets the agreement threshold once the machine exists.
- A benchmark store keyed by commit, which would let CI compare each change with its baseline. Then a
  required check can be reconsidered.
- A regression reaching `main` that the review rule missed. The owner then reviews this decision.
