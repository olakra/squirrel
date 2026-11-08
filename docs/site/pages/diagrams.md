---
title: "Diagrams"
layout: default
---

# Diagrams

Each diagram below was drawn from the source named under it. When the code and a diagram disagree, the code wins and the
diagram is wrong. Diagrams are Mermaid text in this page. The browser draws them from a CDN script when the page loads.
If the script cannot load, the Mermaid source stays on the page.

## System context

Who and what touches the game. Solid arrows exist today. The dashed arrow is planned: the worker does not load
`axiom.wasm` yet.

```mermaid
flowchart LR
    player["Player"]
    subgraph browser["Browser: web/index.html"]
        main["main.js: main thread, canvas hand-off, cursor"]
        input["input.js: gamepad and keyboard intents"]
        worker["worker.js: fixed 60 Hz loop and drawing"]
    end
    wasm["WASM core: axiom.wasm from src/wasm.rs"]
    desktop["Desktop shell: Electron, desktop/"]
    steam["Steam depots: steam/"]

    player -->|keyboard and gamepad| input
    input -->|intents| main
    main -->|intent, canvas, resize| worker
    worker -.->|planned: axiom_* calls| wasm
    desktop -->|runs the web build| main
    steam -->|ships the desktop build| desktop
```

Checked against: `projects/axiom/web/index.html`, `web/main.js`, `web/input.js`, `web/worker.js`,
`projects/axiom/src/wasm.rs`, `projects/axiom/desktop/README.md`, `projects/axiom/steam/README.md`.

## Container view

The Rust crate and its modules. Arrows point from the user to the thing it uses. The crate has no dependency on the web
shell. The dashed arrow is planned.

```mermaid
flowchart TB
    web["web/: JS harness, outside the crate"]
    subgraph crate["Rust crate: projects/axiom/src"]
        wasm["wasm.rs: C-ABI shim, wasm32 or test only"]
        subgraph simulation["simulation"]
            world["world.rs: World"]
            belt["belt.rs: BeltLane, BeltChain"]
            splitter["splitter.rs: Splitter, not in World::step"]
            inserter["inserter.rs: Inserter, not in World::step"]
            chunk["chunk.rs: ChunkGrid, not in World::step"]
        end
        subgraph core["core"]
            ids["ids.rs: ItemId"]
            hash["hash.rs: Fnv1a"]
        end
    end

    wasm --> world
    wasm --> belt
    world --> belt
    world --> hash
    belt --> ids
    splitter --> belt
    splitter --> hash
    inserter --> hash
    chunk --> hash
    web -.->|planned: loads axiom.wasm| wasm
```

Checked against: `projects/axiom/src/lib.rs` (module declarations), the `use crate::` lines in `src/simulation/*.rs`,
`src/core/*.rs` and `src/wasm.rs`, and `src/simulation/world.rs` (`World::step` calls only `BeltChain::tick`).

## Belt tick

One call to `World::step` runs `BeltChain::tick` once. The chain walks its lanes from the last lane to the first, so a
lane frees room before its upstream neighbour hands items over. Each lane advances first, then it may hand its front
item
on.

```mermaid
sequenceDiagram
    participant W as World
    participant C as BeltChain
    participant U as Lane i
    participant D as Lane i+1
    participant S as Sink

    W->>C: tick()
    loop each lane, last to first
        C->>U: advance()
        C->>U: front_ready()
        alt front item not at the exit
            Note over U: nothing to hand over
        else front ready, lane i is not the last
            C->>D: can_accept_rear()
            alt downstream rear has room
                C->>U: pop_front_if_ready()
                C->>D: try_push_rear(item)
            else downstream full
                Note over U: item waits, back-pressure
            end
        else front ready, lane i is the last
            alt sink open
                C->>S: pop_front_if_ready()
                Note over S: sunk += 1
            else sink closed
                Note over U: item waits, back-pressure
            end
        end
    end
```

Checked against: `projects/axiom/src/simulation/world.rs` (`World::step`), `projects/axiom/src/simulation/belt.rs`
(`BeltLane::advance`, `front_ready`, `can_accept_rear`, `try_push_rear`, `pop_front_if_ready`, `BeltChain::tick`,
`set_sink_closed`).

## Inserter state machine

Phases come from `Phase` in `projects/axiom/src/simulation/inserter.rs`. The module comment gives the order:
`SearchingSource -> PickingUp -> RotatingToTarget -> Dropping -> Resetting`. Each timed phase lasts its configured tick
count. The state machine is not wired into `World::step` yet.

```mermaid
stateDiagram-v2
    [*] --> SearchingSource
    SearchingSource --> PickingUp: source.reserve_one() succeeds
    PickingUp --> RotatingToTarget: pick_ticks elapsed, take_reserved()
    RotatingToTarget --> Dropping: rotate_ticks elapsed
    Dropping --> Resetting: drop_ticks elapsed and target.add_one() succeeds
    Resetting --> SearchingSource: reset_ticks elapsed

    note right of SearchingSource
        No free item in the source: stays here.
    end note
    note right of Dropping
        Target full: stays here and keeps the item.
    end note
```

Checked against: `projects/axiom/src/simulation/inserter.rs` (`Phase`, `Inserter::tick`, `search`, `pick_up`, `rotate`,
`drop_into`, `reset`).

## Release pipeline

The release workflow in `.github/workflows/release.yml` runs on a tag that matches `v*.*.*`. Each job needs the one
before
it. The last job waits for a reviewer in the `production` environment.

```mermaid
flowchart LR
    tag["Tag push: v MAJOR.MINOR.PATCH"] --> verify["verify: check-release-tag.sh"]
    verify --> ci["ci: make ci"]
    ci --> build["build: build-wasm.sh, SHA256SUMS"]
    build --> gate{"Approval: production environment"}
    gate -->|approved| release["release: attest and gh release create"]
```

Checked against: `.github/workflows/release.yml` (the `needs:` chain, `environment: production`, and the `permissions`
blocks).
