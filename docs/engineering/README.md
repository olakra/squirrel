# Engineering docs

Standards and decisions that apply across the squirrel monorepo. Project-specific design lives in
each project's `docs/ARCHITECTURE.md`.

| Document | Purpose |
|---|---|
| [code-review.md](code-review.md) | What a reviewer checks, in order, and how to comment. |
| [quality-gates.md](quality-gates.md) | Every CI gate: what it measures, its threshold, and how to fix a failure. |
| [claude-code-setup.md](claude-code-setup.md) | Install Claude Code on Windows and macOS, keep skills and plugins in sync, and sign commits. |
| [engineering-principles.md](engineering-principles.md) | SOLID, domain-driven design, TDD, and the design patterns this codebase uses. |
| [adr/](adr/) | Architecture Decision Records. Start from [adr/0000-template.md](adr/0000-template.md). |
| [branching-strategy.md](branching-strategy.md) | Trunk-based branching: branch types and names, release prep and support branches, and the switch list for a larger team. |
| [change-management.md](change-management.md) | The change lifecycle, risk classes, and the single-owner review exception. |
| [commit-conventions.md](commit-conventions.md) | Conventional Commits types, breaking-change footers, and trailers. |
| [versioning.md](versioning.md) | SemVer per project: versions, the `<project>-vX.Y.Z` tag format, what breaks, and the 0.y.z policy. |
| [release-process.md](release-process.md) | Release runbook around the `make release-*` targets, hotfix flow via support branches, rollback, and cadence. |
| [tag-protection.md](tag-protection.md) | Why release tags are immutable, the two tag rulesets, admin-only tag creation, and immutable releases. |
| [branch-protection.md](branch-protection.md) | Branch rulesets for `main` and `support/**`, required checks, merge settings, and the `production` environment, as UI steps and as a script. |
| [performance-budget.md](performance-budget.md) | The 60 Hz tick budget, the stress scenario, the sandbox baseline, and what the owner must measure on target hardware. |
| [save-format-policy.md](save-format-policy.md) | Save header, version rules, loader rules, and migrations. Policy only; no save code exists yet. |
| [v1-scope.md](v1-scope.md) | Draft v1.0.0 scope: player-visible capabilities and exit criteria. Owner approval required. |
| [../studio/personas.md](../studio/personas.md) | The six studio personas for the tic-tac-toe game: decision rights, questions, artifacts and handoffs. |
| [../studio/art-direction.md](../studio/art-direction.md) | Modern minimalist art direction: colour tokens with contrast ratios, type, spacing, motion and i18n rules. |
| [../studio/icons.md](../studio/icons.md) | The icon set for the tic-tac-toe game and the grid rules each icon follows. |

Accepted decisions:

- [ADR 0001: Monorepo layout](adr/0001-monorepo-layout.md)
- [ADR 0003: Monorepo orchestration (moon)](adr/0003-monorepo-orchestration.md)
- [ADR 0004: Benchmark regression rule as a review rule](adr/0004-benchmark-regression-gate.md)
- [ADR 0005: Keep FNV-1a as the state hash for 0.1.1](adr/0005-state-hash.md)
- [ADR 0006: Tag releases per project and branch from a single trunk](adr/0006-per-project-release-tags.md)

Proposed decisions:

- [ADR 0002: Tech stack, layer by layer](adr/0002-tech-stack.md)
- [Tech stack summary](tech-stack-summary.md): one table of every layer, its status, and the reason.

Reference:

- [build-graph.md](build-graph.md): project dependency graph and task pipeline.

Related:

- [CONTRIBUTING.md](../../CONTRIBUTING.md): setup, branches, pull requests, commit style.
- [.github/workflows/ci.yml](../../.github/workflows/ci.yml): the CI definition these docs describe.
- [.github/workflows/release.yml](../../.github/workflows/release.yml): the release pipeline.
