# Versioning

The axiom crate follows [SemVer 2.0.0](https://semver.org/spec/v2.0.0.html). This page says what
counts as a breaking change for this project and how a version becomes a tag.

## Source of truth

- The version is `version` in `projects/axiom/Cargo.toml`. Nothing else sets it.
- A release tag is `v` followed by that version, for example `v0.1.0`. The tag must equal the
  Cargo version. `release.yml` enforces this in job `verify`, and `make release-check` checks it
  locally.
- Tags live in the repository, so `vX.Y.Z` names the whole repository. When a second project
  ships, its tags move to a project prefix such as `axiom-vX.Y.Z`, and `release.yml` changes with
  them. Do that change before the second project tags anything.
- The version is bumped by hand, in one commit, in the release PR. Commit types do not set it
  automatically. See [release-process.md](release-process.md).

## Public surfaces

A change to any of these is a versioned change and needs a changelog entry.

| Surface | Where it lives | Breaking when |
|---|---|---|
| WASM C-ABI exports | `projects/axiom/src/wasm.rs`, the `#[no_mangle]` functions | A name, argument list, return type, or the meaning of a value changes or is removed. |
| Save format | Not yet defined. It must carry a format version when it is added. | Any file written by an older version cannot be read by the new one, or the reverse is not guaranteed. |
| Determinism (golden hashes) | The fixed-tick state hash checked by the determinism tests | The hash of the same inputs and tick count changes. Every game replay and saved state depends on it. |
| Rust API of the `axiom` crate | `pub` items reachable from `projects/axiom/src/lib.rs` | A `pub` item is removed, renamed, or changes its signature or meaning. |

Internal modules, benchmarks, and the browser harness in `projects/axiom/web/` are not public
surfaces. Their changes are normal changes.

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

While the major version is 0, the project is not yet stable.

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

Do not release 1.0.0 until all of these hold:

1. The save format is defined and has a format version.
2. The C-ABI export list is final for the browser build, and the harness uses only those exports.
3. The golden determinism values have been unchanged across at least two minor releases, or every
   change to them is recorded.
4. The owner has written an ADR that accepts the 1.0.0 surface.

## Tags and pre-releases

- Tags are annotated: `git tag -a vX.Y.Z -m "axiom vX.Y.Z"`. A lightweight tag fails `verify`.
- A tag with a `-` suffix, such as `v0.2.0-rc.1`, is a pre-release. The release is marked as one.
  The suffix is part of the Cargo version too: `version = "0.2.0-rc.1"`.
- A tag is never moved, re-pointed, or deleted. See [tag-protection.md](tag-protection.md).
