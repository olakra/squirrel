# Claude Code setup on Windows and macOS

This guide gets Claude Code working the same way on both development machines. Account-level
skills and plugins from the claude.ai app do not sync into Claude Code automatically, so this
guide keeps the shared parts in git.

## What lives where

| Item | Where it lives | How it reaches both machines |
|---|---|---|
| Project skills | `.claude/skills/<name>/SKILL.md` in this repo | `git pull` |
| Project settings | `.claude/settings.json` in this repo, if added | `git pull` |
| Personal skills | `~/.claude/skills/<name>/SKILL.md` | Copy from a shared folder or dotfiles repo |
| Plugins | Installed with `/plugin` on each machine | Add the same marketplaces and plugins on each machine |
| Conversation history | Local to each machine | Not shared. Use git branches and PRs for work in progress |

## Install Claude Code

Check the current install commands at [docs.claude.com](https://docs.claude.com) before running them.

On Windows (PowerShell):

```powershell
irm https://claude.ai/install.ps1 | iex
```

On macOS (Terminal):

```bash
curl -fsSL https://claude.ai/install.sh | bash
```

If either command fails, install with npm instead (needs Node.js 18 or newer):

```bash
npm install -g @anthropic-ai/claude-code
```

On Windows, Claude Code also needs Git for Windows. Install it with
`winget install --id Git.Git -e`.

Start `claude` once in any folder and sign in with your claude.ai account.

## Sync project skills

Project skills need no extra setup. They live in the repo, so `git pull` on either machine
gets the latest versions. To add one, create `.claude/skills/<name>/SKILL.md` and commit it on a
branch with a PR, like any other change.

## Sync personal skills and plugins

1. On each machine, open `claude` and run `/plugin`. Add the plugin marketplaces you use, then
   install the same plugins on both machines.
2. Keep personal skills in one folder that is part of a git repo, such as a dotfiles repo.
   Copy them into `~/.claude/skills/` on each machine. On Windows that is
   `%USERPROFILE%\.claude\skills\`.
3. Do not commit credentials or tokens to that repo. Claude Code stores sign-in details outside the
   repo.

## Continuing work between machines

- Commit and push work-in-progress on a branch, then pull it on the other machine.
- Use `claude --teleport` to pull a cloud session into the terminal. It needs a clean git state,
  the same repository, and the same claude.ai account.
- Chat history from the claude.ai app stays in the app. It does not move into Claude Code.

## Checks

After setup on each machine, confirm these work:

- `claude` starts and shows you as signed in.
- Project skills from this repo appear when you run `claude` inside the repo.
- Each plugin you use appears in `/plugin`.
