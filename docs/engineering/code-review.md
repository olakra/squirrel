# Code review

A reviewer works through these checks in order. Stop at the first blocking problem and say so.
Earlier checks matter more: a fast, clean diff that breaks determinism is still wrong.

## Order of checks

1. **Correctness and determinism.** The simulation must give the same result on every run and
   platform.
   - No floating-point types in simulation state. Use integers or the fixed-point types the
     architecture names.
   - No iteration over `HashMap` or `HashSet` in simulation code. Iteration order is not
     stable. Use a `Vec`, a sorted structure, or an indexed grid.
   - Time comes from the fixed tick counter, never from the wall clock or the render loop.
2. **Domain model fit.** Names match the ubiquitous language in
   `projects/axiom/docs/ARCHITECTURE.md` (lane, chain, item, offset, tick, and so on). A new
   concept gets a name that the glossary can explain. Do not introduce a second word for an
   existing concept.
3. **SOLID checks.**
   - One reason to change per module. If a module needs editing for both rendering and
     simulation rules, it is doing two jobs.
   - Modules depend on abstractions at their seams (traits or narrow types), not on each other's
     internals.
   - See [engineering-principles.md](engineering-principles.md).
4. **Tests.**
   - Every behaviour change has a test that fails before the change and passes after it.
   - A simulation change adds or updates a determinism test (fixed-tick state hash).
   - Tests check behaviour, not implementation details.
5. **Performance.**
   - The tick path does not allocate. Capacity is reserved at construction.
   - A hot-path change includes a benchmark delta in the PR description. For the stress scenario,
     the rule in [quality-gates.md](quality-gates.md#benchmarks) applies.
6. **Readability.**
   - Functions are short and do one thing. If a function needs a comment to explain its
     sections, split it.
   - Names are explicit. Avoid abbreviations except the ones in the glossary.
   - No clever one-liners. Write the boring version that a new engineer reads in one pass.
7. **CI status.** Every job is green, including coverage, bench smoke, wasm, supply chain, and
   docs. A red job is a blocker, not a note.

## Review size

- Aim for pull requests under about **400 changed lines**. Reviewers catch more defects in small
  diffs.
- Split a larger change into a refactor PR (no behaviour change) and a feature PR. Keep the
  refactor first.
- Generated files and lockfiles do not count against the limit, but say so in the description.

## Comment prefixes

Start every review comment with one prefix so the author knows what they must do.

| Prefix | Meaning | Must the author change it? |
|---|---|---|
| `blocking:` | A correctness, determinism, or policy problem. Merge waits for it. | Yes |
| `question:` | The reviewer does not understand something. Answer it, and maybe add a comment in the code. | Answer it. |
| `nit:` | A small style or naming point. | No. Take it or leave it. |

Example: `blocking: this iterates a HashMap inside World::step, so the state hash is not stable.`

## Approving

- Approve only when every blocking comment is resolved and CI is green.
- A reviewer who did not write the change does the review. CODEOWNERS sets the required reviewer.
