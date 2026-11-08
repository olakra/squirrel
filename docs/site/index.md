---
title: "Overview"
layout: default
---

# Squirrel engineering docs

Squirrel is the monorepo for Oscar's projects. Each project lives under `projects/`. Standards that apply to every
project live under `docs/`. This site renders those Markdown files and their diagrams. The Markdown stays in the
repository, and this site is rebuilt from it on each push to `main`.

## Start here

- [Architecture](axiom/architecture.md): the Axiom simulation design, the constraints in force, and the known gaps.
- [Engineering standards](engineering/index.md): code review, quality gates, principles, change management, releases.
- [Architecture decisions](engineering/adr/index.md): the decisions that shape structure, tooling and contracts.
- [Diagrams](diagrams.md): system context, container view, belt tick, inserter states and the release pipeline.

## Roadmap and specs

- [Roadmap]({{ site.roadmap_url }}): maintained by the planning work. The file opens on GitHub.
- [Specs]({{ site.specs_url }}): source specifications and master prompts live in `docs/specs/`. Design specs will be
  added there.

## Project entry points

- [Axiom README](../../projects/axiom/README.md): build, run and check the simulation.
- [CONTRIBUTING](../../CONTRIBUTING.md): setup, the branch and pull request flow, and commit messages.
