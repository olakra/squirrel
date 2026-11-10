# Quality gates

Each gate is a CI job in [.github/workflows/ci.yml](../../.github/workflows/ci.yml). Run them all
with `make ci`. This page states what each gate measures and how to fix it.

## Gates

| Gate | Command | Measures | Threshold | If it fails |
|---|---|---|---|---|
| fmt | `cargo fmt --all -- --check` | Formatting matches `projects/axiom/rustfmt.toml`. | Zero diffs. | Run `make fmt` and commit the result. |
| clippy | `cargo clippy --all-targets --all-features -- -D warnings` | Lint warnings across lib, tests, and benches. | Zero warnings (warnings are errors). | Fix the lint. If the lint is wrong, add a narrow `#[allow(...)]` with a comment and explain it in the PR. Do not allow it crate-wide. |
| test | `cargo test --all-targets` | Behaviour and determinism on Ubuntu, macOS, and Windows. | All pass on all three OSes. | Run the failing test locally. A test that passes on one OS and fails on another usually means platform-dependent code or float use in the simulation. |
| coverage | `cargo llvm-cov --fail-under-lines 90 --fail-under-functions 90 --fail-under-regions 85` | Share of lines, functions, and regions the tests execute. | Lines 90%, functions 90%, regions 85%. | Add tests for the uncovered path. Do not add tests that only call code without checking results. |
| bench smoke | `cargo bench --bench belt_throughput -- --quick`, then `scripts/ci/check-bench-output.sh` | The benchmark builds, runs, and prints results. Each result is a line that starts with the word "bench". | At least one such line. | Check the bench output in the uploaded `bench-smoke-log` artifact. |
| wasm | `bash scripts/build-wasm.sh` | The WebAssembly build still succeeds. Uploads `axiom.wasm`. | Build succeeds. | Reproduce with `make wasm`. The script owns the build flags. |
| supply chain | `cargo deny check` | Licenses (allow-list), known advisories, yanked and unmaintained crates, banned sources. Policy: `projects/axiom/deny.toml`. | Zero errors. Duplicate versions are warnings. | Upgrade the crate, or justify an exception in `deny.toml` in the PR. |
| docs | `markdownlint-cli2 "**/*.md"` | Markdown structure and style. Rules: `.markdownlint.jsonc`. Excluded: `.markdownlint-cli2.jsonc`. | Zero errors. | Run the linter locally and fix the reported line. |
| tictactoe | `make tictactoe-check`, then `bash web/build.sh` in `projects/tictactoe` | Formatting, clippy (`-D warnings`), and tests for the tic-tac-toe crates, then the browser bundle builds. Uploads `tictactoe-site`. | Zero diffs, zero warnings, all tests pass, bundle builds. | Run `make tictactoe-check`, then `make tictactoe-web` (needs wasm-bindgen-cli 0.2.100). |
| scripts-test | `make scripts-test` | Every script in `scripts/ci/` and `scripts/release/` parses (`bash -n`) and passes its `--self-test`. | All exit 0. | Run `make scripts-test` and read the first failing self-test. |
| commit-lint | `scripts/ci/check-commit-messages.sh` | Every commit subject in the pull request, and the pull request title, follow [commit-conventions.md](commit-conventions.md). | Zero bad subjects. | Fix the title, or reword the commit. Run `make commit-lint` locally. |
| changelog-check | `scripts/ci/check-changelog.sh` | Per project: a pull request that changes a project's watched paths adds an entry under `[Unreleased]` in that project's changelog. The paths are in [versioning.md](versioning.md#changelogs). | An entry for each project touched, or the `no-changelog` label with a reason. | Add the entry to the right project's changelog. Run `make changelog-check` locally. |
| branch-name | `scripts/ci/check-branch-name.sh` | The head branch name follows [branching-strategy.md](branching-strategy.md#naming-rules). | A valid name. | Rename the branch with `git branch -m <new-name>`, push it, and open a new pull request. Run `make branch-check` locally. |
| js-test | `bun test` in `projects/axiom/web`, and `bun run test` in `projects/axiom/desktop` | Input translation (deadzone, edge-triggered buttons, D-pad repeat, frame-time cap) and the desktop `prepare-web` script. | All pass. | Run `make js-test` locally. Read the failing assertion before changing the expected value. |

The js-test gate is a CI job in [.github/workflows/steam-build.yml](../../.github/workflows/steam-build.yml),
not in `ci.yml`. It is also not part of `make ci`, because `ci.yml` does not install Bun. Run it with
`make js-test`. Add it to `make ci` once `ci.yml` installs Bun 1.4.4.

## What the coverage gate means

Coverage is the **quality target**. It is a floor, not a grade.

- It proves the tests **run** the code. It does not prove the tests **check** the right results.
  Review checks that.
- Line and function coverage are at 90%. Region coverage is 85%, because region coverage counts
  every branch within a line and is stricter.
- The gate runs over the whole crate, so a new module with no tests lowers the total.

## What the lint gate means

"Zero clippy warnings" is enforced by `-D warnings` on the command line in CI. The file
`projects/axiom/clippy.toml` sets thresholds. It does not turn warnings into errors on its own.

- `cognitive-complexity-threshold = 15` applies only if the `cognitive_complexity` lint is enabled.
  That lint is in clippy's nursery group, so it is off by default and CI does not enable it.
  Until someone adds `-W clippy::cognitive_complexity` to the CI command, complexity is a
  reviewer check (see [code-review.md](code-review.md)).
- `too-many-arguments-threshold = 5` is active because `too_many_arguments` is on by default.

## Benchmarks

The spec asks for a stress benchmark (100,000 items on 10,000 active belt segments) and for
performance to hold at 60 ticks per second. Benchmarks are tracked as trends, not as a hard CI gate.

- **CI checks that benchmarks run** (bench smoke). It does not judge their timing. Shared CI
  runners vary too much from run to run to fail a build on a timing difference.
- **Regression rule.** A change that makes the stress scenario more than **10% slower** fails
  review until the PR explains why the cost is worth it. The reviewer enforces this rule. No
  automated comparison exists yet. The decision and the review steps are in
  [ADR 0004](adr/0004-benchmark-regression-gate.md). The baseline is in
  [performance-budget.md](performance-budget.md).
- **Hot-path changes** include a before and after benchmark result in the PR description. Run them
  on the same machine.
- A future step is a benchmark store keyed by commit. Then a regression can fail CI automatically.
  It is not built.

## Coverage numbers in the spec

The brief for this repo asks for a ">90% of benchmarks" target. The specs do not contain that
target. The specs only ask for a benchmark suite. The ">90%" in this repo is the coverage floor
above. It is a test-coverage threshold, not a count of benchmarks that pass.

## Tooling not yet verified in CI

These gates were written without being run in CI. Before relying on them, check:

- `cargo-deny`: confirm the `deny.toml` keys still match the installed cargo-deny version.
  Confirm the crate's `UNLICENSED` license is handled (see the comment in `deny.toml`).
- `cargo-llvm-cov`: confirm the thresholds pass on the current test suite.
- `scripts/build-wasm.sh`: this script is still being written. The wasm job calls it as-is.
