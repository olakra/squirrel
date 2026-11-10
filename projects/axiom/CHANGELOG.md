<!-- markdownlint-disable-file MD024 -->
# Changelog

All notable changes to the axiom crate are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[SemVer](https://semver.org/spec/v2.0.0.html). The policy is in
[docs/engineering/versioning.md](../../docs/engineering/versioning.md).

## [Unreleased]

## [0.1.2] - 2026-11-09

Milestone 1: the simulation core, its WASM export layer, the browser item feed, the first
tic-tac-toe game, and the repository governance. This is the first published version. The
number 0.1.1 was assigned to this milestone but never tagged.

### Added

- Simulation items are drawn each frame from a WASM snapshot (`axiom_snapshot_items`,
  `axiom_item_id`, `axiom_item_lane`, `axiom_item_offset`). The renderer draws them as
  instanced quads.
- `F` feeds one item into the head of the chain.
- Pure mapping from a snapshot to renderer instances (`web/items.js`), with tests.
- Belt model: fixed-capacity ring-buffer lanes with O(1) rear push and front pop, checked
  against a reference implementation.
- Splitter: deterministic round-robin routing, with handling for blocked outputs.
- Inserter: a discrete state machine with source reservation.
- Chunk grid: 32 by 32 chunks with sleep and wake, and sorted iteration.
- WASM C-ABI exports: `axiom_init`, `axiom_step`, `axiom_feed`, `axiom_state_hash_lo`,
  `axiom_state_hash_hi`, `axiom_sunk`, `axiom_tick_count`.
- Tic-tac-toe (`projects/tictactoe`, not part of the axiom crate): rules core with 19 tests,
  a web front end with screen-reader labels and a live status line, and CI for the Rust to
  WebAssembly build.

### Changed

- Coverage gate tests call the FFI exports directly, so the export layer counts toward the
  function threshold.
- Commit-lint runs on pull requests only. Push runs on `main` re-check history that predates
  the rule.

### Fixed

- The tic-tac-toe win line stayed visible after Reset. SVG has no `hidden` property, so the
  JavaScript toggle had no effect. The layer now uses the hidden attribute and CSS.
- The coverage gate failed because the new WASM exports had no tests. FFI export tests now
  call them directly.
- CI run 42512418750 failed on commit-lint for push runs and on Markdown and changelog checks.
  Fixed in #30.
- Markdown lint failures: bare URLs, list numbering, and line length in the READMEs and the
  steam notes.
- Changelog check failed on Axiom source changes without an entry. Entries now exist.

### Documentation

- Claude Code setup guide for Windows and macOS (`docs/engineering/claude-code-setup.md`).
- Studio personas, art direction, and icon spec for tic-tac-toe (`docs/studio/`).
- Release notes for this version (`docs/releases/v0.1.2.md`) and the roadmap.

### Governance

- Repository ignores agent worktrees under `.claude/worktrees/`.

[Unreleased]: https://github.com/olakra/squirrel/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/olakra/squirrel/releases/tag/v0.1.2
