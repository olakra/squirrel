# Engineering docs

Standards and decisions that apply across the squirrel monorepo. Project-specific design lives in
each project's `docs/ARCHITECTURE.md`.

| Document | Purpose |
|---|---|
| [code-review.md](code-review.md) | What a reviewer checks, in order, and how to comment. |
| [quality-gates.md](quality-gates.md) | Every CI gate: what it measures, its threshold, and how to fix a failure. |
| [engineering-principles.md](engineering-principles.md) | SOLID, domain-driven design, TDD, and the design patterns this codebase uses. |
| [adr/](adr/) | Architecture Decision Records. Start from [adr/0000-template.md](adr/0000-template.md). |

Accepted decisions:

- [ADR 0001: Monorepo layout](adr/0001-monorepo-layout.md)

Related:

- [CONTRIBUTING.md](../../CONTRIBUTING.md): setup, branches, pull requests, commit style.
- [.github/workflows/ci.yml](../../.github/workflows/ci.yml): the CI definition these docs describe.
