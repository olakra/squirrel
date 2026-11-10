# Versioning

Every project that ships follows [SemVer 2.0.0](https://semver.org/spec/v2.0.0.html). Each project
versions on its own. This page says what counts as a breaking change and how a version becomes a tag.

## Source of truth

The projects are listed in one registry, `scripts/release/projects.sh`. Every release script, check,
and workflow reads it.

| Project | Version source | Mirrors | Changelog |
|---|---|---|---|
| axiom | `projects/axiom/Cargo.toml`, `[package]` | `projects/axiom/desktop/package.json` | `projects/axiom/CHANGELOG.md` |
| tictactoe | `projects/tictactoe/Cargo.toml`, `[workspace.package]` (members use `version.workspace = true`) | `projects/tictactoe/web/package.json` | `projects/tictactoe/CHANGELOG.md` |

- The version source is the only place that sets a version. The mirrors must equal it.
  `make release-prepare` updates both, and `make release-check PROJECT=<project>` checks them.
- The version is bumped by hand, in one commit, in the release PR. Commit types do not set it
  automatically. See [release-process.md](release-process.md).
- Run `make versions` to see each project's version and latest tag.

## Tag format

- A release tag is `<project>-v` followed by that project's version, for example `axiom-v0.2.0` or
  `tictactoe-v0.1.0`. The tag must equal the version source. `release.yml` enforces this in job
  `verify`, and `make release-check` checks it locally.
- **Legacy tag.** `v0.1.2`, with no prefix, is axiom's first release. It stays forever and is never
  moved. Every new tag uses the prefix. The rule in earlier versions of this page, "move to a project
  prefix before the second project tags anything", is now done.
- Tic-tac-toe has never been tagged on its own. Its first release is `tictactoe-v0.1.0`.

## Public surfaces

A change to any of these is a versioned change and needs a changelog entry in that project's
changelog.

### axiom

| Surface | Where it lives | Breaking when |
|---|---|---|
| WASM C-ABI exports | `projects/axiom/src/wasm.rs`, the `#[no_mangle]` functions | A name, argument list, return type, or the meaning of a value changes or is removed. |
| Save format | Not yet defined. It must carry a format version when it is added. | Any file written by an older version cannot be read by the new one, or the reverse is not guaranteed. |
| Determinism (golden hashes) | The fixed-tick state hash checked by the determinism tests | The hash of the same inputs and tick count changes. Every game replay and saved state depends on it. |
| Rust API of the `axiom` crate | `pub` items reachable from `projects/axiom/src/lib.rs` | A `pub` item is removed, renamed, or changes its signature or meaning. |

Internal modules, benchmarks, and the browser harness in `projects/axiom/web/` are not public
surfaces. Their changes are normal changes.

### tictactoe

| Surface | Where it lives | Breaking when |
|---|---|---|
| Rust API of the rules core | `pub` items reachable from `projects/tictactoe/core/src/lib.rs` | A `pub` item is removed, renamed, or changes its signature or meaning. |

Before 1.0.0, the web bundle (`projects/tictactoe/web/`) and its i18n keys are not a contract. Their
changes are normal changes, but they still need a changelog entry when players can see them.

### Changelogs

`changelog-check` checks every project. A pull request that changes a project's watched paths needs an
entry under `[Unreleased]` in that project's changelog, or the `no-changelog` label with a reason.

- axiom watches `projects/axiom/src/**` and `projects/axiom/Cargo.toml`.
- tictactoe watches `projects/tictactoe/Cargo.toml`, `{core,web}/src/**`, `{core,web}/Cargo.toml`, and
  `web/www/**`, all under `projects/tictactoe/`.

Repository-wide governance changes, such as docs and CI, need no project changelog.

## What counts as breaking

Breaking changes are the ones that need a `BREAKING CHANGE:` footer and a `!` after the type.
Examples:

- Removing or renaming an exported C-ABI function, or changing its argument types.
- Changing the state hash for the same inputs. Even a fix that corrects an earlier mistake
  breaks golden values, so the changelog must say so.
- Changing a save format without reading the old format.
- Changing the order in which the simulation iterates its items, where that order is observable
  in the state hash.

Not breaking:

- A new export, a new `pub` item, or a new optional field that old data can omit.
- A performance change that leaves every hash and every output the same.
- A fix that changes behaviour only for inputs that were already wrong, if the old output was not
  a contract. Say so in the changelog under "Fixed".

## Pre-1.0 policy (0.y.z)

While the major version is 0, the project is not yet stable. These rules apply to each project.

- A **minor** bump (`0.y` to `0.(y+1)`) may break a public surface. Each break must appear in the
  changelog under a "Changed" or "Removed" heading, and must start with the word **Breaking**.
- A **patch** bump (`0.y.z` to `0.y.(z+1)`) contains fixes only. It must not change a public
  surface, and it must not change a golden hash. A fix that must change a hash is a minor bump.
- Every release, including a patch, has a changelog section. The section is the release notes.

## Bump rules

| Change | Pre-1.0 bump | Post-1.0 bump |
|---|---|---|
| Fix, no public surface change | patch | patch |
| New feature, backward compatible | minor | minor |
| Breaking change to any public surface | minor (with **Breaking** in the changelog) | major |
| Fix that changes a golden hash | minor (with **Breaking** in the changelog) | major |
| Documentation or pipeline change only | no release | no release |

## Moving to 1.0.0

Do not release axiom 1.0.0 until all of these hold:

1. The save format is defined and has a format version.
2. The C-ABI export list is final for the browser build, and the harness uses only those exports.
3. The golden determinism values have been unchanged across at least two minor releases, or every
   change to them is recorded.
4. The owner has written an ADR that accepts the 1.0.0 surface.

## Tags and pre-releases

- Tags are annotated and signed. `make release-tag PROJECT=<project>` creates them with `git tag -s`,
  with the message `<project> vX.Y.Z` followed by the changelog section. A lightweight tag fails
  `verify`.
- A tag with a `-` suffix, such as `axiom-v0.3.0-rc.1`, is a pre-release. The release is marked as
  one. The suffix is part of the version too: `version = "0.3.0-rc.1"`.
- A pre-release is never marked "Latest" on GitHub. Only stable axiom releases are.
- A tag is never moved, re-pointed, or deleted. See [tag-protection.md](tag-protection.md).
