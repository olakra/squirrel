<!-- markdownlint-disable-file MD024 -->
# Changelog

All notable changes to the axiom crate are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[SemVer](https://semver.org/spec/v2.0.0.html). The policy is in
[docs/engineering/versioning.md](../../docs/engineering/versioning.md).

## [Unreleased]

Nothing yet.

## [0.1.1] - 2026-11-08

Milestone 1: the simulation core, its WASM export layer, and the repository governance.

### Added

- Belt model: fixed-capacity ring-buffer lanes with O(1) rear push and front pop, checked
  against a reference implementation.
- Splitter: deterministic round-robin routing, with handling for blocked outputs.
- Inserter: a discrete state machine with source reservation.
- Chunk grid: 32 by 32 chunks with sleep and wake, and sorted iteration.
- WASM C-ABI exports: `axiom_init`, `axiom_step`, `axiom_feed`, `axiom_state_hash_lo`,
  `axiom_state_hash_hi`, `axiom_sunk`, `axiom_tick_count`.
- Governance: CI gates for format, clippy, tests on three operating systems, coverage, bench
  smoke, WASM build, supply chain, and Markdown. Engineering standards, ADRs, and the
  Conventional Commits rules.

[Unreleased]: https://github.com/olakra/squirrel/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/olakra/squirrel/releases/tag/v0.1.1
