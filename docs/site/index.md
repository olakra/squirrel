---
title: "Overview"
layout: default
---

## Squirrel, an indie game studio

Squirrel is a small indie game studio, and this monorepo is where its games are built. Each game lives under
`projects/`. Standards that apply to every project live under `docs/`. This site renders those Markdown files and
their diagrams. The Markdown stays in the repository, and the site is rebuilt from it on each push to `main`.

## Our games

- **Axiom**: a deterministic 2D factory simulation, with a dependency-free Rust core that runs in the browser as
  WebAssembly and on the desktop and Steam on top of that build. See the
  [Axiom README](../../projects/axiom/README.md) and the [architecture](axiom/architecture.md).
- **Tic-tac-toe**: a small browser game. The rules are plain Rust built to WebAssembly. See the
  [Tic-tac-toe README](../../projects/tictactoe/README.md).

## Start here

- [Architecture](axiom/architecture.md): the Axiom simulation design, the constraints in force, and the known gaps.
- [Engineering standards](engineering/index.md): code review, quality gates, principles, change management, releases.
- [Architecture decisions](engineering/adr/index.md): the decisions that shape structure, tooling and contracts.
- [Diagrams](diagrams.md): system context, container view, belt tick, inserter states and the release pipeline.

## How we work

- [Branching strategy](engineering/branching-strategy.md): branches, pull requests and how a release is cut.
- [Code review](engineering/code-review.md): what a reviewer checks and how feedback is given.
- [Quality gates](engineering/quality-gates.md): the checks every change passes before it merges.
- [Release process](engineering/release-process.md): versioning, changelogs and tags.
- [CONTRIBUTING](../../CONTRIBUTING.md): setup, the branch and pull request flow, and commit messages.

## Roadmap and specs

- [Roadmap]({{ site.roadmap_url }}): maintained by the planning work. The file opens on GitHub.
- [Specs]({{ site.specs_url }}): source specifications and master prompts live in `docs/specs/`. Design specs will be
  added there.
