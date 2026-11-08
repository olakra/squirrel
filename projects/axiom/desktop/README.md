# Axiom desktop shell

An Electron shell that runs the bundled `web/` build as a native app on Windows, macOS and Linux. Steam
and the Steam Deck use the same build. The reasoning is in [ADR-0001](ADR-0001-desktop-shell.md).

**Status:** this shell has not been built or run. Packages could not be installed in the environment that
wrote it. The first `bun install` and `bun run dist:*` must happen on a machine with network access.

## Layout

| Path | Purpose |
|---|---|
| `main.js` | Main process: window, `axiom://` scheme, isolation headers, CSP, network block. |
| `preload.js` | Sandboxed preload. Exposes `window.axiomDesktop` with read-only values. No IPC. |
| `electron-builder.yml` | Packaging. Every target is `dir`. |
| `scripts/prepare-web.mjs` | Copies `../web` to `app-web/` for packaging. Fails if `axiom.wasm` is missing. |
| `scripts/stage-depot.mjs` | Copies one platform's output to `dist/staging/<platform>/` for Steam. |
| `test/*.test.mjs` | `bun test` suites for the scripts above. |
| `package.json` | Bun 1.4.4 (`packageManager`), Electron and electron-builder ranges, and the scripts Bun runs. |
| `bun.lock` | Exact versions. Must be committed. Not yet created (see below). |

## Run and package

Run these from this folder, on a machine with network access. Install Bun 1.4.4 first (see bun.sh).

```bash
bun install                 # first time: creates bun.lock, then commit it. CI uses --frozen-lockfile.
bun run start               # development: loads ../web directly, no copy step
bun run dist:linux          # packages dist/linux-unpacked (also dist:win, dist:mac)
bun run test                # Bun tests for the scripts. Needs no install and no network.
```

Node.js 22 or newer is still needed. Electron's install script runs on Node, and the packaged
app runs on Electron's own Node. Bun does not replace either.

### Why Bun, and what it does not replace

Bun installs packages, runs the `package.json` scripts and runs the tests. It does not replace the Chromium
runtime. Electron bundles Chromium and Node for the app itself, so the packaged game still runs
on Electron, not Bun. `trustedDependencies` in `package.json` lists `electron`, because Bun does
not run dependency install scripts unless they are listed there. Electron's install script
downloads the Electron binary.

The scripts stay `.mjs`. They use only `node:fs` and `node:path`, and Bun runs them unchanged. A
rewrite to TypeScript would add a build step for no runtime gain, and this shell is not large
enough to need one. `steam/render_vdf.py` stays Python.

Packaging needs `../web/axiom.wasm`. Build it first with `projects/axiom/scripts/build-wasm.sh`.

Electron version: `^40.0.0` is the range this shell was written against. Before the first release,
bump to the current supported major, and check the Electron release timeline. An unsupported major
gets no security fixes.

## Behaviour

- Window opens at 1280x800 (the Steam Deck's native size). Minimum 960x600. Resizable.
- F11 or Alt+Enter toggles fullscreen. Escape stays with the game, as the cancel key.
- Gamepad input comes from the page's Gamepad API. No flags are needed. A controller is detected after
  its first button press, which is a Chromium rule.
- Navigation outside `axiom://app`, new windows and webviews are blocked.
- Every permission request is denied. The network is blocked except for the app's own origin.
- Background throttling is off, so the simulation keeps running when the window loses focus.

## Known gaps

- **Linux sandbox on SteamOS.** Chromium's sandbox needs unprivileged user namespaces, or a setuid
  `chrome-sandbox`. A Steam depot cannot set the setuid bit. Test the Linux build on a Deck first. Only
  if it aborts with a sandbox error, consider the `--no-sandbox` launch option, and record the reason in
  `steam/README.md` and ADR-0001.
- **Code signing.** macOS is unsigned (`identity: null`). Windows is unsigned. Signing and notarization
  come before any public release, and Steam may require it for macOS (verify).
- **Placeholder identifiers.** `appId` in `electron-builder.yml` is `org.example.axiom`. Replace it.
- **Icons.** No icon files are included. Add them to `build/` before release.
