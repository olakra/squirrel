# Save format and versioning policy

- **Status:** Policy, not implemented. No save code exists in `projects/axiom/src/`.
- **Date:** 2026-11-08
- **Tracks:** issue #19. The save row in [tech-stack-summary.md](tech-stack-summary.md) is Deferred.

## Scope

This policy covers any file the game writes to disk to restore simulation state. A save must
round-trip exactly: a loaded state has the same state hash as the state that was saved. Any change to
the format is a versioned change, and [versioning.md](versioning.md) governs it.

## Header

Every save starts with this header. Integers are little-endian. No field is optional.

| Offset | Size | Field | Rule |
|---|---|---|---|
| 0 | 4 bytes | Magic | A fixed ASCII value, chosen when the format is first written. It identifies the file type. |
| 4 | 4 bytes | Format version | `u32`, starting at 1. Bumped by any change to the layout or meaning of a field. |
| 8 | 1 byte | Hash algorithm | Names the content hash. Ids are assigned in this file when save code lands. |
| 9 | N bytes | Content hash | Length set by the algorithm. Covers header bytes 0 to 8 and the whole body. |
| after header | variable | Body | Fields in a fixed order, written by hand. No serde on simulation state. |

The body records the state hash at save time. The loader checks it after loading (rule 5 below).

The content hash detects truncated or corrupted files. It does not defend against tampering. See
[ADR 0005](adr/0005-state-hash.md).

## Versions

- The first version is 1. It is the first release that writes saves.
- A version bump is required for any change to the layout, the meaning of a field, or the hash
  algorithm.
- A version number is never reused.
- Each version ships with a changelog entry that names the change.

## Loader rules

1. Read the magic first. Unknown magic is refused.
2. Read the version. A version newer than the loader's is refused, with a message that names both
   versions. A loader never guesses at a layout.
3. Verify the content hash before decoding the body. On mismatch, refuse. Never load part of a file.
4. For an older version, run the migration chain one step at a time, up to the current version.
5. After loading, recompute the state hash. It must equal the state hash recorded in the body.

## Compatibility

- **Newer game reads older save:** supported, through migrations. Every version from 1 stays migratable.
  Dropping a migration would strand players' saves, so migrations are removed only with a changelog
  entry. Issue #19 suggested supporting only the last N versions. This policy keeps all migrations
  instead. The owner confirms the choice.
- **Older game reads newer save:** not supported. The older build refuses the file with the version
  error. No downgrade promise is made.

## Breaking save changes

A change is breaking when a loader must do something different with an existing file. In practice:

- Any change to the layout or the meaning of a field. It bumps the version and needs a migration.
  Before 1.0.0 it is a minor bump with **Breaking** in the changelog ([versioning.md](versioning.md)).
- Any change to the hash algorithm.
- Any change that makes the same save load to a different state, even a fix. The changelog says so.
- Any change to iteration order that the state hash can observe ([versioning.md](versioning.md)).

A change that leaves the format version unchanged is not a save change. Adding a field is a layout
change, so it bumps the version.

## Migration policy

- Each version bump has exactly one migration, from version N to version N+1. Code never skips a
  version.
- A migration converts data in version N form to version N+1 form.
- Each version has a golden save file committed to the repository. The test records the expected
  state hash after load. The path is proposed as `projects/axiom/tests/golden/saves/`, and it is
  created with the first save code.
- The migration test loads each golden file, migrates it to the current version, and checks the
  expected state hash. A missing golden file fails the test.
- A golden file is never regenerated to match new code. If one fails, the code is wrong, unless an ADR
  records the decision to change the expected result.

## Tests required before save code merges

1. Round trip: save, load, and get an identical state hash for the 1,000-tick scenario in issue #19.
2. Golden load for every version in the repository.
3. Refusal cases: a flipped body byte, a truncated file, an unknown magic, and a newer version.
4. Migration chain: version 1 to the current version.

## Open decisions

- The magic value and the hash algorithm ids. Both are chosen when save code lands.
- Whether a save records `ChunkGrid` sleep state. This depends on issue #1.
- The save location in the Deck checklist (item 19). It links here once save code exists.
