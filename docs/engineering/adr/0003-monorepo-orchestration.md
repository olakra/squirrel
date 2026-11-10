# ADR 0003: Monorepo build and task orchestration

- **Status:** Proposed
- **Date:** 2026-11-08
- **Deciders:** repository owner (olakra)

## Context

The repository now holds five kinds of build input: a Rust crate (`projects/axiom`), a Bun and
JavaScript browser harness (`projects/axiom/web`), an Electron shell that declares Bun 1.4.4 as its
package manager (`projects/axiom/desktop`), Python with the standard library only
(`projects/axiom/steam/render_vdf.py`), and shell scripts (`scripts/ci/*.sh`,
`projects/axiom/scripts/build-wasm.sh`). Markdown under `docs/` is linted by a pinned
markdownlint-cli2 call.

The facts that shape the decision:

- **Orchestration is a Makefile and three workflows that repeat its commands.** `make ci` runs eight
  gates in a fixed order (`Makefile`). `ci.yml`, `release.yml` and `steam-build.yml` restate the
  same commands, and the comments say "keep the two in step". Drift has already started:
  `ci.yml` uses `dtolnay/rust-toolchain@stable`, `release.yml` installs nightly without a pin, and
  `steam-build.yml` pins Bun through its own `BUN_VERSION` variable.
- **Real dependency edges exist, and no tool declares them.** `scripts/build-wasm.sh` writes
  `projects/axiom/web/axiom.wasm`. `web/main.js` fetches `./axiom.wasm`.
  `desktop/scripts/prepare-web.mjs` copies `web/` into `desktop/app-web` and fails without the wasm.
  `desktop/scripts/stage-depot.mjs` needs the desktop output. `render_vdf.py` needs a staging folder.
  `docs/engineering/engineering-principles.md` names crate paths such as `projects/axiom/src/simulation`.
  Today CI runs every gate on every push. That is correct but does not scale.
- **Lockfiles are only partly in place.** `projects/axiom/Cargo.lock` exists. The desktop package
  declares Electron and electron-builder but no `bun.lock` was present when this was written, so a
  frozen install cannot run yet.
- **Release rules constrain caching.** Tags are never moved or deleted, and the release is built
  only after the `verify` job checks the tag (`release.yml`, `tag-protection.md`). A cached release
  artifact must not be served without that check.
- **Constraints of this change.** One maintainer. Another agent is migrating `web/`, `desktop/` and
  `steam/` to Bun at the same time, so those directories are not edited here. The build sandbox has
  no network access to package registries, so no orchestrator binary can be installed or run.

### Requirements, in order of weight

The orchestrator must guarantee the following. Each requirement has a weight in the scoring below.

1. A correct dependency graph across languages: wasm depends on the crate, the web bundle on the
   wasm, the desktop package on the web bundle, the depot on the desktop package, docs on the code
   they describe.
2. Pinned tool versions, so a run on a laptop and a run in CI use the same Bun, Node and Python.
3. Affected-only builds and tests, so a change in one project does not rebuild the others.
4. Remote caching that never serves a release artifact for a tag without verification.
5. Fit for one maintainer, with low ongoing cost.
6. Ease of local verification: the same commands run locally and in CI.
7. Maturity and ecosystem.
8. Migration cost from the Makefile.
9. Exit cost: how hard it is to abandon the tool later.

The orchestrator does not replace the native tools. Cargo builds the crate, Bun runs the web and
desktop code, and Python and the shell run the rest. The orchestrator sequences them, caches what
they produce, and runs only what a change affects.

## Options considered

Scores are judgments from 1 (poor) to 5 (strong) for each criterion. The weights sum to 100.
The weighted score is the sum of weight times score, divided by 5, so the maximum is 100.

| Criterion | Weight |
|---|---|
| Correct cross-language dependency graph | 20 |
| Hermetic, pinned tool versions | 15 |
| Affected-only builds and tests | 15 |
| Remote caching that is release-safe | 10 |
| Fit for one maintainer | 15 |
| Ease of local verification | 10 |
| Maturity and ecosystem | 5 |
| Migration cost from the Makefile | 5 |
| Exit cost | 5 |

| Option | Graph | Hermetic | Affected | Remote cache | Maintainer | Local verify | Maturity | Migration | Exit | Score |
|---|---|---|---|---|---|---|---|---|---|---|
| **moon** (chosen) | 4 | 3 | 4 | 3 | 5 | 4 | 3 | 4 | 5 | **78** |
| Bazel with bazelisk | 5 | 5 | 4 | 5 | 1 | 2 | 5 | 1 | 2 | 72 |
| Nx or Turborepo | 2 | 2 | 5 | 4 | 3 | 3 | 4 | 3 | 3 | 62 |
| Pants | 3 | 3 | 4 | 3 | 4 | 3 | 4 | 2 | 3 | 66 |
| Buck2 | 4 | 4 | 3 | 4 | 2 | 2 | 2 | 1 | 2 | 60 |
| Task (go-task) | 2 | 1 | 2 | 1 | 5 | 5 | 5 | 5 | 5 | 59 |
| Keep the Makefile | 1 | 1 | 1 | 1 | 5 | 4 | 5 | 5 | 5 | 50 |

### moon

A Rust binary that runs tasks across projects and languages. Each project declares its tasks,
inputs, outputs and dependencies in a `moon.yml`. The workspace file lists the projects. The toolchain
file pins tool versions, including Bun and Node, and Python.

- **Strengths.** One binary. Explicit cross-project task dependencies (`deps`) and project
  dependencies (`dependsOn`). Tool version management in the repository. Affected-task detection
  against the VCS base. Tasks are shell commands, so a task keeps working if the tool is removed.
- **Weaknesses.** Younger than Bazel and Pants, so the schema and the CLI change more often
  (verify the version the team pins). Hermeticity is convention plus pinned tools, not a sandbox.
  Remote caching depends on a hosted service or a self-hosted server whose availability and
  terms must be checked (verify). Rust is not managed as a moon toolchain in this change, so the
  crate's toolchain comes from `rust-toolchain.toml` (see the follow-ups).

### Bazel (with bazelisk)

The industry standard for large polyglot repositories. `rules_rust` is mature, `rules_js` and
`rules_ts` cover JavaScript, and the remote execution API gives remote caching and execution.
Versions are pinned through `MODULE.bazel` and `.bazelversion`, and bazelisk selects the Bazel
version. Hermeticity is strongest of all the options.

- **Why it is not chosen.** The learning curve is steep, and the configuration for four languages
  is large for one maintainer. The first build needs network access to fetch rules and the
  toolchains, and this sandbox cannot run it, so the configuration could not be checked here. Moving
  every script into Starlark is the largest migration of the options. Its main benefit, bit-exact
  hermetic builds, does not change the current release risk, which is a checksum and an attestation
  of a build that is already reviewed (`release.yml`).
- **Revisit when** the crate splits into several Rust crates that need per-crate caching, or when
  the release must be reproducible bit for bit (see "What would change this decision").

### Nx and Turborepo

JavaScript-first task runners with strong affected detection and remote caching. Rust is handled
only by community plugins (verify). Both expect a root `package.json` workspace. The Bun migration is
editing `package.json` files now, so adopting either would add conflicts to that work.

### Pants

Python-first, with good affected detection (changed-file queries) and remote caching. As far as
known, Pants has no first-party Rust backend (verify). Its JavaScript backend has been experimental
(verify). Rust would therefore be a shell wrapper with no crate-level graph. Pants would fit a
Python-heavy repository better than this one.

### Buck2

Rust-based, with Rust rules in its prelude and remote execution. JavaScript support depends on
community rules (verify). Its open-source documentation and Windows support are thinner than the
Bazel and moon docs (verify). The setup cost for one maintainer is high for what it adds here.

### Task (go-task) or just

Honest baseline. Task's `sources` and `generates` fields give local up-to-date checks by checksum,
and it can call the same commands. It has no cross-project graph by default, no tool version
management and no remote cache. It is the right fallback if moon fails the verification spike.

### Keep the Makefile

Costs nothing now, and every target is a shell command. It has no graph, no affected detection and
no cache. The CI workflows keep restating its commands, which is the drift this decision removes.

## Decision

We will use **moon** to orchestrate builds, tests and release tasks across the repository. The
workspace is `.moon/workspace.yml`, tool versions are in `.moon/toolchain.yml`, and each project's
tasks are in `orchestration/projects/<id>/moon.yml`. The tasks are `check`, `test`, `build`,
`package`, and `release-verify`. The `publish` step stays in `release.yml`, outside the orchestrator,
because it needs the `production` approval and the attestation token.

The seven projects and their edges are in `docs/engineering/build-graph.md`.

### Why moon, tied to the criteria

- **Graph (20).** Edges are declared as `dependsOn` between projects and `deps` between tasks.
  Each edge names a real file or command. The graph is acyclic (checked, see below).
- **Hermetic tools (15).** Bun, Node and Python are pinned in `.moon/toolchain.yml`. Bun 1.4.4 is the
  version already named in `desktop/package.json`. Rust stays pinned by a file the crate owns (see
  follow-ups). This scores below Bazel because nothing sandboxes the tasks.
- **Affected-only (15).** Each task lists its inputs, so a change outside those inputs does not
  re-run it. Affected detection is a moon feature to verify in the spike (see "Not verified here").
- **Remote caching (10).** Available as an option, but not required for the decision. Tag builds
  must not read the cache (see the guarantees below).
- **One maintainer (15), local verification (10).** One binary. The task commands are the same
  shell commands the Makefile already runs, so a developer can run them without moon.
- **Migration (5), exit (5).** Each Makefile target maps to one task (see the migration plan).
  Leaving moon means deleting `.moon/` and `orchestration/`, and the Makefile or Task file
  takes over the same commands.

### Guarantees and what enforces them

| What must hold | Mechanism in this change | Status |
|---|---|---|
| The graph is declared and acyclic | `dependsOn` and `deps` in each `moon.yml` | Declared. Acyclic and all paths exist, checked by a parser script (see below). |
| Tool versions are pinned | `.moon/toolchain.yml`: Bun 1.4.4, Node 22, Python 3.13 | Partly. Rust is not pinned yet. |
| Lockfiles are honored | `cargo` reads `Cargo.lock`. `desktop:fetch` uses `bun install --frozen-lockfile` | Blocked. `bun.lock` is not committed for desktop. |
| Affected-only runs | Inputs and file groups on every task | Declared, not exercised. |
| A release is never served from cache without verification | Tag builds bypass the cache. `axiom-wasm:package` depends on `ci-scripts:release-verify` | Policy and one dependency. CI wiring is not done. |
| Outputs that depend on secrets are not cached | `axiom-steam:package` sets `cache: false` | Declared. |
| Network only in declared fetch steps | `axiom-desktop:fetch` is the only fetch task | Not met. `docs-site:check` runs `npx`, which downloads a tool. Recorded as a follow-up. |

### Migration plan from the Makefile

**Phase 0 (this change).** Add the configuration and this ADR. Nothing is wired into CI. The
Makefile and the workflows are unchanged.

**Phase 1 (spike, needs registry access).** Install the pinned moon version. Run `moon check --all`,
`moon project-graph` and `moon run :check`. Fix schema key names where moon reports them. Compare
the result of `moon run` with `make ci` on the same commit, gate by gate, before anything else
changes. Confirm three things: affected detection, the cache bypass flag, and the `command` and
`script` semantics used here.

**Phase 2 (Makefile as thin wrappers).** Each Make target calls the matching task. The Makefile keeps
the same names, so developers' habits do not change.

| Makefile target | moon task |
|---|---|
| `fmt-check`, `lint` | `axiom-core:check` |
| `test` | `axiom-core:test` |
| `cov` | new `axiom-core:coverage` (cargo llvm-cov thresholds) |
| `bench` | new `axiom-core:bench` (belt_throughput, check-bench-output) |
| `wasm` | `axiom-wasm:build` |
| `deny` | new `axiom-core:deny` (cargo deny check) |
| `docs` | `docs-site:check` (scope to be decided: the root `**/*.md` glob is wider than `docs/`) |
| `ci` | the `check` and `test` tasks of every project, run with `moon run :check :test` |
| `commit-lint`, `changelog-check` | new `ci-scripts` tasks (they need the base ref as an argument) |
| `release-check PROJECT=axiom` | `ci-scripts:release-verify` (axiom only; other projects run the make target) |
| `scripts-test` | `ci-scripts:test` |

Two behaviors change. First, moon runs tasks in parallel by default, but the Makefile runs gates in
a fixed order so failures come in a predictable sequence. The gates are independent except where a
task `deps` entry says otherwise, so the new order is acceptable. Each task reports its own failure.
Second, `make ci` no longer runs the gates in one shell, so a failing gate's output is reported per
task.

**Phase 3 (CI).** The jobs in `ci.yml` call `moon run <task>` for their gate. The three-OS test matrix
stays. The job names do not change, so branch protection's required checks keep working. A remote
cache, when adopted, is read-write for branches and never read for tags.

**Phase 4 (release).** The `verify` job in `release.yml` calls `ci-scripts:release-verify` with the
tag, in CI mode. The `build` job calls `axiom-wasm:build` and `axiom-wasm:package` with the cache
bypassed. The `release` job, which holds the approval and the publish step, does not change.

**Phase 5 (Makefile retired).** Once CI runs through moon, the Makefile targets stay as aliases for
developers, or are removed. `scripts/ci/*.sh` stay where they are.

**Ownership moves.** When each owner of `web/`, `desktop/` and `steam/` finishes the Bun migration,
that owner moves the matching `orchestration/projects/<id>/moon.yml` next to the code and changes one
line in `.moon/workspace.yml`. The task definitions do not change.

## Consequences

- **Easier.** Each task runs the same way locally and in CI. The graph is explicit, and a change
  to the crate shows which projects it affects. Tool versions are in one file. Each edge has a
  file to point to.
- **Harder.** Developers need the moon binary for the orchestrated path. The schema is new to the
  team. Until Phase 2 lands, two sources of truth exist, the Makefile and the moon tasks.
- **Hermeticity is a convention, not a sandbox.** A task can read a file outside its inputs and
  still be cached. Reviews must check the inputs of every new task. Bazel would enforce this.
- **Ruled out for now.** Bazel, Pants and Buck2 are not adopted. The revisit triggers are below.
- **Follow-ups.**
  1. Add `projects/axiom/rust-toolchain.toml` pinned to the version CI uses. This edits the crate
     folder, so the crate owner should make the change.
  2. Commit `projects/axiom/desktop/bun.lock`, and `projects/axiom/web` if it gains dependencies.
     The Bun migration owner does this.
  3. Replace `npx markdownlint-cli2` in `docs-site:check` with a pinned tool.
  4. Add the coverage, bench and deny tasks to `axiom-core` (Phase 2 table).
  5. Add a check that `docs/engineering/build-graph.md` matches the workspace graph. Today it is
     written by hand.
  6. Decide whether `dist/` at the repository root is the release staging folder for `package` (it is
     gitignored today).

### What would change this decision

- The crate splits into several Rust crates that change independently, and build time becomes a
  problem. Revisit Bazel with `rules_rust`.
- The release must be reproducible bit for bit, or a reviewer must verify the build from source.
  Revisit Bazel, which is hermetic by design.
- Pants gains a first-party Rust backend, and the Python surface grows beyond `render_vdf.py`.
  Revisit Pants.
- The moon spike in Phase 1 fails: the schema does not match, affected detection does not work as
  described, or the cache bypass for tags cannot be set. Fall back to Task with the same task names,
  which keeps the migration cost low.
- A second maintainer joins and needs a stricter sandbox. Revisit Bazel, since its cost is then
  shared.

### Not verified here

This sandbox has no moon binary and no network access to package registries (npm returned network
errors when tested). So:

- **Schema keys are unverified.** The keys used (`$schema`, `dependsOn`, `fileGroups`, `inputs`,
  `outputs`, `deps`, `script`, `command`, `options.runFromWorkspaceRoot`, `options.cache`, the
  `vcs` and `projects` workspace keys, and the toolchain file name `toolchain.yml`) follow moon's
  documentation as recalled. The spike must check each one.
- **Affected detection, the cache bypass for tags, and the `@group(...)` syntax are unverified.**
- **Versions are unverified.** Bun 1.4.4 comes from the repository. Node 22 and Python 3.13 are
  marked "verify" in `.moon/toolchain.yml`.
- **What was verified.** Every YAML file parses with `yaml.safe_load`. The project graph is
  acyclic. Every project path exists, and every task dependency names a project and task that exist.
  Every input and output path names an existing directory, except outputs and lockfiles that the
  build creates or the Bun owner commits: `axiom.wasm`, `dist/`, `desktop/dist/` and `desktop/bun.lock`.
- **Bazel and Pants claims** are from general knowledge and are marked "verify" where unsure.
- **Markdown lint** was not run, because the linter needs the npm registry.
- **Mermaid** in `build-graph.md` was not rendered. It was checked by reading.
- **ADR 0002** did not exist in `docs/engineering/adr/` when this was written, so it was not read
  or edited. If it lands with a tech-stack decision, this ADR's tool choices must be reconciled
  with it.
