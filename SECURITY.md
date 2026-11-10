# Security policy

## Reporting a vulnerability

Do not open a public issue, pull request, or discussion for a security problem.

Report it privately with GitHub private vulnerability reporting: open the repository's **Security** tab, choose
**Report a vulnerability**, and describe the problem. Only the maintainer can see the report.

Private vulnerability reporting must be turned on in the repository settings (Settings, then Code security). The
maintainer enables it. If the **Report a vulnerability** button is missing, the feature is not on yet. Tell the
maintainer by opening a normal issue that says only "please enable private vulnerability reporting". Do not put the
details in that issue.

Include what you need us to reproduce it: the affected component (for example the desktop shell, the WASM build, or
the CI workflows), the commit SHA, the steps, and the impact as you see it.

## Supported versions

Each project is released on its own. The latest published release of each project is supported. Older releases
are not patched; the fix is to upgrade.

| Project | Supported |
|---|---|
| axiom | `v0.1.2`, released 2026-11-09, and `main` |
| tictactoe | `main` only. It has no release yet. Its first will be `tictactoe-v0.1.0` |

Fixes land on `main`. When a released version needs a fix before the next normal release, it goes out as a patch
release. If `main` already has unreleased work, the patch is cut from a support branch (see
[docs/engineering/branching-strategy.md](docs/engineering/branching-strategy.md)).

## Response expectations

This is a one-maintainer project. There is no service-level agreement, and no bug bounty. What we aim for:

- An acknowledgement of your report, usually within a week.
- An assessment: whether it is a vulnerability, how severe it is, and which versions it affects.
- A fix for confirmed issues, prioritised by severity. Timing depends on complexity. Critical issues come first.
- Credit in the release notes if you want it. Tell us the name or handle to use.

We will keep you informed of progress. If we decide not to fix something, we will say why.

## Scope

In scope: the code in this repository, including the desktop shell (`projects/axiom/desktop`), the web build
(`projects/axiom/web`), the Steam packaging (`projects/axiom/steam`), the tic-tac-toe game (`projects/tictactoe`),
and the CI and release workflows (`.github/workflows`).

Out of scope: third-party services and platforms (Steam, GitHub, Electron, Chromium) except where our configuration
makes the problem worse. Report those upstream.

Do not test against systems you do not own. Do not access other users' data.
