# ADR 0005: Keep FNV-1a as the state hash for 0.1.1

- **Status:** Accepted
- **Date:** 2026-11-08
- **Deciders:** repository owner (olakra), through the owner's delegation for non-owner tasks

## Context

The state hash fingerprints the simulated world. The determinism tests and the benchmark's
`state_hash` lines use it to show that two runs produced the same state.

- The code uses FNV-1a 64 in `projects/axiom/src/core/hash.rs`. It has no dependency. Its test checks
  the published FNV-1a vector for the input `a`.
- The specs disagree. The rust-wasm spec's determinism test allows FNV-1a or BLAKE3, over 1,000 ticks.
  The studio spec requires 50,000 ticks with an identical BLAKE3 state hash. ADR 0002 (requirement R3)
  cites the studio spec.
- [tech-stack-summary.md](../tech-stack-summary.md) lists BLAKE3 as the target for the state digest.
  That row comes from ADR 0002, which is still Proposed.
- BLAKE3 is not available from this sandbox. crates.io is not reachable, so a new dependency cannot be
  fetched or checked against [deny.toml](../../../projects/axiom/deny.toml).
- Every recorded hash changes when the algorithm changes. That includes the determinism test values
  and the benchmark's `state_hash` lines.
- [versioning.md](../versioning.md) says a change to the state hash for the same inputs is breaking.
  Under the pre-1.0 policy, it needs a minor bump with a **Breaking** changelog entry.
- FNV-1a is not a cryptographic hash. It detects divergence between runs. It does not resist deliberate
  tampering.
- No save code exists yet. A save format needs a content hash, which is a separate decision. See
  [save-format-policy.md](../save-format-policy.md).

Options considered:

1. **Keep FNV-1a.** No dependency, and the tests run offline. Chosen for 0.1.1.
2. **Move to BLAKE3 now.** Blocked: the registry is unreachable from this sandbox, and every expected
   hash changes.
3. **Hybrid: FNV-1a for parity, BLAKE3 for save checksums.** This is option 3 in issue #9. It is
   deferred until save code is written. The save format names its hash algorithm in the file header,
   so the choice can be made then.

## Decision

We will keep FNV-1a 64 as the state hash for 0.1.1. We will move to BLAKE3 in one release, as a single
change, when a dependency is allowed. That change updates every expected hash value, and its changelog
entry is marked **Breaking**.

## Consequences

- Easier: the simulation core stays dependency-free, and `cargo test` runs offline.
- Harder: the studio spec's 50,000-tick BLAKE3 requirement is not met by the current code. The v1.0.0
  scope lists this as an open question. See [v1-scope.md](../v1-scope.md).
- Follow-up: when the switch happens, it is one pull request. Every expected hash value and the
  benchmark's expected output change in that pull request. Known gap 2 in
  [ARCHITECTURE.md](../../../projects/axiom/docs/ARCHITECTURE.md) points here.
- Any save file written with FNV-1a must be readable after a later switch. The save format header
  names its hash algorithm, so a migration can handle it.

## What would change the decision

- The owner allows a crates.io dependency. That removes the blocker, but the decision still needs the
  switch above, in its own release.
- Save code needs an integrity check. Then BLAKE3 may be added for saves, which is option 3.
- An external consumer depends on the hash values.
- The v1.0.0 scope adopts the 50,000-tick BLAKE3 test as an exit criterion.
