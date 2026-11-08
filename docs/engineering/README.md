# Engineering docs

Standards and decisions that apply across the squirrel monorepo. Project-specific design lives in
each project's `docs/ARCHITECTURE.md`.

| Document | Purpose |
|---|---|
| [code-review.md](code-review.md) | What a reviewer checks, in order, and how to comment. |
| [quality-gates.md](quality-gates.md) | Every CI gate: what it measures, its threshold, and how to fix a failure. |
| [engineering-principles.md](engineering-principles.md) | SOLID, domain-driven design, TDD, and the design patterns this codebase uses. |
| [adr/](adr/) | Architecture Decision Records. Start from [adr/0000-template.md](adr/0000-template.md). |
| [change-management.md](change-management.md) | The change lifecycle, risk classes, and the single-owner review exception. |
| [commit-conventions.md](commit-conventions.md) | Conventional Commits types, breaking-change footers, and trailers. |
| [versioning.md](versioning.md) | SemVer for the axiom crate: what breaks, the 0.y.z policy, and how tags map to versions. |
| [release-process.md](release-process.md) | Release runbook, hotfix flow, rollback, and cadence. |
| [tag-protection.md](tag-protection.md) | Why release tags are immutable, the tag ruleset, and immutable releases. |
| [branch-protection.md](branch-protection.md) | Branch rules, required checks, and the `production` environment, as UI steps and as a script. |

Accepted decisions:

- [ADR 0001: Monorepo layout](adr/0001-monorepo-layout.md)
- [ADR 0003: Monorepo orchestration (moon)](adr/0003-monorepo-orchestration.md)

Reference:

- [build-graph.md](build-graph.md): project dependency graph and task pipeline.

Related:

- [CONTRIBUTING.md](../../CONTRIBUTING.md): setup, branches, pull requests, commit style.
- [.github/workflows/ci.yml](../../.github/workflows/ci.yml): the CI definition these docs describe.
- [.github/workflows/release.yml](../../.github/workflows/release.yml): the release pipeline.
