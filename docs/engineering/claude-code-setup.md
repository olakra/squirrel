# Claude Code setup on Windows and macOS

This guide gets Claude Code working the same way on both development machines. Account-level
skills and plugins from the claude.ai app do not sync into Claude Code automatically, so this
guide keeps the shared parts in git.

## What lives where

| Item | Where it lives | How it reaches both machines |
|---|---|---|
| Project skills | `.claude/skills/<name>/SKILL.md` in this repo | `git pull` |
| Project settings | `.claude/settings.json` in this repo | `git pull` |
| Agent rules | `CLAUDE.md` at the repo root | `git pull` |
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

## Project settings and agent rules

- `.claude/settings.json` holds the project permissions. Its deny rules stop Claude Code from pushing
  tags, force-pushing, and deleting tags, on every machine and for every session in this repo.
- `CLAUDE.md` holds the rules agents follow here: work on a branch, open a pull request, never push a
  tag, and never merge their own pull request without the owner. See
  [branching-strategy.md](branching-strategy.md).

Change either file through a pull request, like any other change.

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

## Signed commits

Every commit on `main` is signed. GitHub signs the squash merge, and the rulesets require signatures
(see [branch-protection.md](branch-protection.md)). Sign your own commits and tags too. The release
targets sign tags with `git tag -s`, so the owner's machine must have signing set up.

### How each Claude Code surface signs

- **Claude Code CLI, desktop, and IDE extensions** run your local `git`. They use your global git
  config, so they sign with the key set up below. Nothing else to configure.
- **Claude Code on the web** signs its commits on the platform. They show as Verified on GitHub.

### Set up SSH signing

Use the SSH key you already have, or create one with `ssh-keygen -t ed25519`. `user.email` must be a
verified address on your GitHub account. The GitHub noreply address works.

On Windows (PowerShell):

```powershell
git config --global gpg.format ssh
git config --global user.signingkey "$env:USERPROFILE\.ssh\id_ed25519.pub"
git config --global commit.gpgsign true
git config --global tag.gpgsign true
git config --global gpg.ssh.allowedSignersFile "$env:USERPROFILE\.ssh\allowed_signers"
$key = Get-Content "$env:USERPROFILE\.ssh\id_ed25519.pub"
"$(git config user.email) $key" | Out-File -Encoding ascii -Append "$env:USERPROFILE\.ssh\allowed_signers"
```

On macOS (Terminal):

```bash
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/id_ed25519.pub
git config --global commit.gpgsign true
git config --global tag.gpgsign true
git config --global gpg.ssh.allowedSignersFile ~/.ssh/allowed_signers
echo "$(git config user.email) $(cat ~/.ssh/id_ed25519.pub)" >> ~/.ssh/allowed_signers
```

Each line of `allowed_signers` is `<email> <key type> <key>`. Git uses it to verify signatures
locally.

### Register the key on GitHub as a signing key

A signing key is separate from an authentication key. Add the same public key again with the type
`signing`, even if it is already registered for authentication. The commands are the same on both
systems:

```sh
gh auth refresh -h github.com -s admin:ssh_signing_key
gh ssh-key add ~/.ssh/id_ed25519.pub --type signing --title "<machine> signing"
```

On Windows, give the path as `$env:USERPROFILE\.ssh\id_ed25519.pub`. Until the key is registered,
GitHub shows your signed commits and tags as unverified (`unknown_key`). That is the state of the
`v0.1.2` tag.

### Verify

- `git log --show-signature -1` prints `Good "git" signature` for your last commit.
- `git tag -v <tag>` checks a signed tag.
- On GitHub, your commits show the "Verified" badge.

## Checks

After setup on each machine, confirm these work:

- `claude` starts and shows you as signed in.
- Project skills from this repo appear when you run `claude` inside the repo.
- Each plugin you use appears in `/plugin`.
- `git log --show-signature -1` shows a good signature, and GitHub shows your commits as Verified.
