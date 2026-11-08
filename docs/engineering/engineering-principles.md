# Engineering principles

These are the working rules for squirrel, with examples from the Axiom simulation. The
[code review checklist](code-review.md) applies them. Where a rule is not yet followed in the code,
the status line says so.

## SOLID

- **Single responsibility.** One reason to change per module. A module that parses input and also
  steps the simulation has two reasons to change.
- **Open/closed.** Add behaviour by adding a type that implements an existing trait, not by
  editing a `match` in the core loop. Example: a new inserter kind is a new state-machine
  implementation, not another branch in `World::step`.
- **Liskov.** Any implementation of a trait keeps the trait's promises. A routing strategy must
  still give deterministic output for the same input.
- **Interface segregation.** Small traits. A renderer that only needs item positions should not
  depend on the whole world.
- **Dependency inversion.** Modules depend on traits at the seams between bounded contexts, not on
  each other's internal types.

## Domain-driven design

### Bounded contexts

| Context | Owns | Lives in |
|---|---|---|
| Simulation core | Items, lanes, chains, ticks, state hash. Pure logic, no I/O. | `projects/axiom/src/simulation`, `src/core` |
| Rendering | Turning simulation state into sprites. Reads state, never writes it. | Planned. Not yet in the crate. |
| Web shell | Browser glue: canvas, workers, wasm bindings, input. | `projects/axiom/web` (JS), wasm bindings planned |

The rule: the simulation core never imports from rendering or the web shell. Dependencies point
inward, toward the core.

### Ubiquitous language

Use these words in code, tests, and docs. Do not invent synonyms. The authoritative list is
`projects/axiom/docs/ARCHITECTURE.md`. Terms in use now:

- **Item**: a unit moving on a belt. Identified by `ItemId`.
- **Lane**: one belt track. Holds items in ascending offset order.
- **Chain**: lanes linked so items can move from one to the next. Identified by position in the
  world.
- **Offset**: a position along a lane, measured in integer units.
- **Spacing**: the minimum gap kept between adjacent items.
- **Tick**: one fixed 60 Hz simulation step. Not a frame.
- **State hash**: the FNV-1a hash of the simulation state, used by determinism tests.

## Test-driven development

Work in red-green-refactor steps:

1. **Red.** Write the test for the behaviour you want. Run it and watch it fail, for the reason you
   expect.
2. **Green.** Write the smallest change that passes it.
3. **Refactor.** Clean up with the tests still green. Do not change behaviour in this step.

Two kinds of test matter most here:

- **Behaviour tests** for each rule: items compress when blocked, a splitter alternates outputs.
- **Determinism tests** for every simulation change. Run a fixed layout for a fixed number of ticks
  and compare the state hash with a recorded value. A change that alters the hash must say so in
  the PR, and the expected value must be updated on purpose.

```rust
#[test]
fn blocked_front_item_rests_at_spacing_cap() {
    let mut lane = BeltLane::new(/* length */ 40);
    lane.push_rear(ItemId(1), 0);
    lane.tick_blocked(); // downstream refuses
    assert_eq!(lane.front_offset(), 40 - ITEM_SPACING);
}
```

Status: the belt lane and world determinism tests exist. Splitter, inserter, and chunk tests are in
progress. The example above is illustrative. Check the crate for the current method names before
copying it.

## Design patterns in this codebase

Use a pattern when it removes a real problem. Do not use one because it is a pattern.

| Pattern | Used for | When to use it | Status |
|---|---|---|---|
| Newtype | `ItemId(u16)`, `LaneIndex(u32)` in `src/core/ids.rs` | Two integer ids must not be swapped by mistake. | In use. |
| State machine | Inserter: `SearchingSource`, `PickingUp`, `RotatingToTarget`, `Dropping`, `Resetting` | An entity moves through named phases with rules for each transition. Write transitions as one `match`. | In progress (`src/simulation/inserter.rs`). |
| Strategy | Splitter output routing | Several interchangeable routing rules that must all be deterministic. Pick one behind a trait. | In progress (`src/simulation/splitter.rs`). |
| Ring buffer | Belt lane storage | A lane needs O(1) insertion at the rear and no allocation. This is planned to replace the current O(N) rear insert (ARCHITECTURE.md, gap 1). | Planned. |
| Repository-like grid | Chunk storage (32x32) | Look up entities by location without scanning the world. The grid owns storage; callers ask it questions. | In progress (`src/simulation/chunk.rs`). |

Short example of the newtype pattern from `src/core/ids.rs`:

```rust
#[repr(transparent)]
pub struct ItemId(pub u16);
pub struct LaneIndex(pub u32);
// fn take(lane: LaneIndex) cannot be called with an ItemId: the compiler stops the mix-up.
```

## Determinism rules (summary)

These apply to every simulation change. They are checked in review first.

- No `f32` or `f64` in simulation state.
- No iteration over `HashMap` or `HashSet` in simulation code.
- No allocation in the tick path. Reserve capacity at construction.
- No wall-clock time in simulation code.
- Any `unsafe` block states its contract in a `# Safety` section.
