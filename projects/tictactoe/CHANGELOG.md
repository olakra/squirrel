<!-- markdownlint-disable-file MD024 -->
# Changelog

All notable changes to the tic-tac-toe game (`projects/tictactoe`) are recorded here. The format
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[SemVer](https://semver.org/spec/v2.0.0.html). The policy is in
[docs/engineering/versioning.md](../../docs/engineering/versioning.md).

Releases are tagged `tictactoe-vX.Y.Z`.

## [Unreleased]

## [0.1.0] - 2026-11-10

The first standalone release of the tic-tac-toe web game. The release files are the browser bundle
(`tictactoe-web-0.1.0.tar.gz`, the `web/www` folder with the built WebAssembly) and `SHA256SUMS`. The
same code was also included in the repository tag `v0.1.2`, cut before projects had their own tags.

### Added

- Rules core (`tictactoe-core`): turns, wins, draws, and refused moves, with 19 tests. No
  dependencies and no I/O.
- Web front end (`tictactoe-web`): the core built to WebAssembly and a 3 by 3 board in the browser.
  Each cell has a screen-reader label, and a live status line announces turns, results, and
  refused moves.
- Every visible string comes from a translation file (`web/www/i18n/en.json`).
- A line is drawn across the winning cells.
- CI builds and tests both crates and the browser bundle in the `tictactoe` job.

### Fixed

- Reset clears the win line.

[Unreleased]: https://github.com/olakra/squirrel/compare/tictactoe-v0.1.0...HEAD
[0.1.0]: https://github.com/olakra/squirrel/releases/tag/tictactoe-v0.1.0
