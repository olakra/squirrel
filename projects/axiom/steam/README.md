# Steam packaging

This folder holds the steamcmd templates, the token renderer, and the launch-options reference for
shipping Axiom on Steam. It contains no real App IDs, depot IDs, usernames or tokens. Those values
come from Steamworks and from GitHub Actions secrets.

## Files

| File | Purpose |
|---|---|
| `app_build.vdf` | App build template. Tokens such as `${STEAM_APP_ID}` are filled at render time. |
| `depot_build_windows.vdf`, `depot_build_linux.vdf`, `depot_build_macos.vdf` | One depot template per OS. Each maps its OS folder. |
| `render_vdf.py` | Fills the tokens from environment variables. It writes nothing if any token has no value or an ID is not numeric. |
| `launch-options.template.json` | Reference values for the Steamworks Launch Options page. |
| `deck-verified-checklist.md` | Steam Deck Verified requirements to check before submitting. |

Do not run steamcmd on the templates directly. Render them first.

## Owner steps (one time, before the first upload)

These steps happen in Steamworks and GitHub. Nothing in this repository can do them.

1. Create a Steamworks partner account at partner.steamgames.com. Complete the identity, tax and
   banking forms. Steam will not publish without them.
2. Pay the Steam Direct fee for the app. Check the current amount in the Steamworks documentation.
3. Create the app in Steamworks. Steamworks assigns the App ID. Copy it from the app page. Do not
   make one up.
4. On the app's Depots page, create three depots, one each for Windows, macOS and Linux. Steamworks
   assigns each depot ID. Set each depot's operating system, and check the Steam Deck notes in the
   Steamworks documentation for how the Linux and Windows depots are used on the Deck (verify).
5. Set the launch options per OS, using `launch-options.template.json` as the reference.
6. Create a dedicated Steam account for uploads. Turn on Steam Guard. Give it permission to upload
   builds for this app (verify the permission name in the Steamworks Users and Permissions docs).
7. Log in once with steamcmd as that account and complete the Steam Guard prompt. steamcmd then
   keeps a login token in its `config/config.vdf` file. The path depends on the steamcmd version
   (verify). This file is a credential. Do not commit it or paste it into chat.
8. Add these GitHub Actions secrets in the repository settings. Store the names and values as
   described. Do not store the account password, because the workflow logs in with the token only.

| Secret name | What to store |
|---|---|
| `STEAM_USERNAME` | The upload account's username. |
| `STEAM_CONFIG_VDF` | The `config.vdf` from step 7, base64-encoded on one line (for example `base64 -w0 config.vdf` on Linux). |
| `STEAM_APP_ID` | The App ID from step 3. |
| `STEAM_DEPOT_WINDOWS_ID` | The Windows depot ID from step 4. |
| `STEAM_DEPOT_MACOS_ID` | The macOS depot ID from step 4. |
| `STEAM_DEPOT_LINUX_ID` | The Linux depot ID from step 4. |

The Steam Web API key is not needed for uploads, so skip it for now.

9. When the login token expires, or the upload account's password changes, repeat step 7 and
   replace `STEAM_CONFIG_VDF`.

## What the workflow does

`.github/workflows/steam-build.yml` runs on tags `v*`, and on manual dispatch for builds only.

1. A WASM job builds `axiom.wasm` with `scripts/build-wasm.sh`.
2. A matrix job runs on Ubuntu, Windows and macOS. Each job packages the desktop shell with
   electron-builder in `--dir` mode, stages the output with `stage-depot.mjs`, and uploads it as a tar.
   The tar keeps file modes and symlinks, which the macOS app bundle needs.
3. The upload job runs only on tags, and only when `STEAM_USERNAME` is set. It downloads the three
   depot tars, renders the templates with `render_vdf.py`, and runs steamcmd with
   `+run_app_build`. The build goes to a preview. It is not set live. Set a branch live by hand in
   Steamworks after you test the preview.

Secrets are passed through environment variables, not interpolated into commands. The workflow masks
the upload account name and deletes the decoded login token at the end of the job.

## Local dry run (no upload)

Run these from `projects/axiom/desktop`. Use your own values in place of the placeholders. Do not
paste real values into shared logs.

```bash
bun install --frozen-lockfile
bun scripts/prepare-web.mjs --allow-missing-wasm    # local test only; release builds need web/axiom.wasm
bunx electron-builder --linux --dir --publish never
bun scripts/stage-depot.mjs linux
STEAM_APP_ID=<app id> STEAM_DEPOT_WINDOWS_ID=<id> STEAM_DEPOT_LINUX_ID=<id> STEAM_DEPOT_MACOS_ID=<id> \
  AXIOM_STAGING_DIR=$PWD/dist/staging python3 ../steam/render_vdf.py --out "$PWD/dist/steam"
```

Without `--allow-missing-wasm`, `prepare-web.mjs` stops when `web/axiom.wasm` is missing. Run
`bun run dist:linux` for the same packaging steps, with that check on.

## Before the first upload

- Replace the placeholder `appId` in `desktop/electron-builder.yml` with your own reverse-DNS
  identifier. It is an internal bundle ID, not a Steam ID.
- Add app icons and Steam store art. Steamworks requires store art, and the app needs its own icon.
- Sign and notarize the macOS build with an Apple Developer ID. Check whether Steamworks requires
  this for the macOS depot (verify).
- Commit `desktop/bun.lock`. The workflow runs `bun install --frozen-lockfile`, which fails without it.
- Test each depot on real hardware from a preview branch before setting anything live.
- Confirm the Linux build starts on SteamOS. See the sandbox note in `desktop/README.md`.

## Next step: Steamworks SDK

Achievements, cloud saves and the Steam overlay need the Steamworks SDK. Put that code in a separate
module that is loaded only in the Steam build. Keep the desktop shell free of Steam code, so it
still builds and runs without the SDK.
