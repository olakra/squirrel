<!-- markdownlint-disable-file MD024 -->
# Changelog

All notable changes to the tic-tac-toe game (`projects/tictactoe`) are recorded here. The format
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[SemVer](https://semver.org/spec/v2.0.0.html). The policy is in
[docs/engineering/versioning.md](../../docs/engineering/versioning.md).

Releases are tagged `tictactoe-vX.Y.Z`.

## [Unreleased]

Everything below ships in the first standalone release, `tictactoe-v0.1.0`. The same code was also
included in the repository tag `v0.1.2`, which was cut before projects had their own tags.

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

[Unreleased]: https://github.com/olakra/squirrel/commits/main/projects/tictactoe
