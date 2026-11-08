# v1.0.0 scope

- **Status:** Draft, owner approval required. Not approved.
- **Date:** 2026-11-08
- **Tracks:** issue #21, which stays open until the owner approves this scope.

## Purpose

Milestone 4 (v1.0.0) has no scope in the repository, so no issues can be planned against it. This draft
defines v1.0 by what a player can do, and by exit criteria that can be checked. Items marked "open
question" need the owner's answer before approval. Each in-scope item links to an issue or is marked
"to be filed".

## In scope: what a player can do

| # | Capability | What the player does | Status today | Tracked by |
|---|---|---|---|---|
| S1 | Belts | Lays belt lanes and chains. Items move, and they back up when blocked. | Simulated and tested. Not drawn. | #13, #14 |
| S2 | Splitters and inserters | Splits a belt in two. Moves items from a belt into a machine or a chest. | Units implemented and tested. Not wired into the world. | #11, #13 |
| S3 | Producer and consumer | A producer adds items to a belt. A consumer takes them off. | Not started. Items enter by a feed call, and leave through a sink. | To be filed |
| S4 | Save and load | Saves a factory, quits, and loads it later with the same state. | Policy only. | #19, [save-format-policy.md](save-format-policy.md) |
| S5 | Browser play | Opens the web build and plays with no install. | Harness only. | #12, #14 |
| S6 | Steam play | Plays the Steam build on Windows, macOS, Linux and Steam Deck. | Desktop shell built. Not on Steam. | #15, #16, #17, #18, #20 |
| S7 | Controller-only play | Does every action with a controller. | Done in code. Owner test pending. | #16 |

## Out of scope (proposed)

- Multiplayer. The spec describes a single-player game.
- Mod support, localization, achievements, and cloud saves. The specs do not ask for them.
- Native hosts (Tauri, or native Rust with wgpu) and a WebGPU renderer. ADR 0002 defers these.
- Power grids, trains, chemical processing, thermal systems, and the orbital mass driver. The studio
  spec's core loop names them. This draft places them after v1.0.0. See open question 3.

## Exit criteria

All of these must hold for v1.0.0.

**Engine and performance**

- E1. 60 UPS is sustained on target hardware with the stress scenario in
  [performance-budget.md](performance-budget.md). The owner chooses the item count (open question 1).
- E2. 60 FPS on integrated graphics at 1080p, with frame time recorded (issue #13).
- E3. Memory stays at or below 256 MB after 10 hours of play (studio spec). No soak test exists yet.
- E4. Load to play takes 3.5 s or less on broadband. The WASM build is 8 MB or less compressed (studio
  spec). Not measured yet.

**Determinism and saves**

- D1. A 50,000-tick run gives an identical state hash on Ubuntu, macOS and Windows (studio spec). The
  hash algorithm must be decided first (open question 2).
- D2. The save round trip and the golden-file tests in [save-format-policy.md](save-format-policy.md) pass.

**Platform**

- P1. Zero-install play works in Chrome, Edge, Safari and Firefox (studio spec). Safari and Firefox
  have not been tested from this environment (ADR 0002).
- P2. Every row of the [Steam Deck checklist](../../projects/axiom/steam/deck-verified-checklist.md) is
  done, or has a stated reason to skip it. A preview build has been tested on a real Deck.
- P3. The macOS build is notarized and the Windows build is signed (issue #18).
- P4. Electron is on a supported major version (issue #20).

**Quality and release**

- Q1. The quality gates pass on `main`, including coverage (issue #5).
- Q2. Every item in [versioning.md](versioning.md), "Moving to 1.0.0", holds, and the owner writes the
  ADR that accepts the 1.0.0 surface.
- Q3. The owner-only items are complete: the release environment (issue #10), Steamworks (issue #15),
  and signing (issue #18).

## Open questions for the owner

1. Is the v1.0.0 engine bar 200,000 active items at 60 UPS (studio spec), or the 100,000-item stress
   scenario in the benchmark? The benchmark measures 100,000 only.
2. Does v1.0.0 keep FNV-1a, or move to BLAKE3 before release? ADR 0005 keeps FNV-1a for 0.1.1, but the
   studio spec requires a BLAKE3 hash over 50,000 ticks.
3. Are power, trains, chemical processing, and the mass driver in v1.0.0, or after it?
4. Are mobile browsers in scope? The studio spec names them as a target.
5. Is one producer and one consumer enough for the first playable slice, or does v1.0.0 need a full
   recipe chain?
6. Is manual recovery (harvesting scrap and ore by hand) in v1.0.0? The studio spec's core loop starts
   there.

## Approval

- [ ] Owner approves this scope, with any changes listed above answered.
- Approved by: ______ on ______
