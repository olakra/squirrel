# Axiom desktop shell

An Electron shell that runs the bundled `web/` build as a native app on Windows, macOS and Linux. Steam
and the Steam Deck use the same build. The reasoning is in [ADR-0001](ADR-0001-desktop-shell.md).

**Status:** this shell has not been built or run. Packages could not be installed in the environment that
wrote it. The first `npm ci` and `npm run dist:*` must happen on a machine with network access.

## Layout

| Path | Purpose |
|---|---|
| `main.js` | Main process: window, `axiom://` scheme, isolation headers, CSP, network block. |
| `preload.js` | Sandboxed preload. Exposes `window.axiomDesktop` with read-only values. No IPC. |
| `electron-builder.yml` | Packaging. Every target is `dir`. |
| `scripts/prepare-web.mjs` | Copies `../web` to `app-web/` for packaging. Fails if `axiom.wasm` is missing. |
| `scripts/stage-depot.mjs` | Copies one platform's output to `dist/staging/<platform>/` for Steam. |
| `package.json` | Pinned ranges for Electron and electron-builder. |

## Run and package

Run these from this folder, on a machine with network access.

```bash
npm ci                      # first time: creates package-lock.json, then commit it
npm start                   # development: loads ../web directly, no copy step
npm run dist:linux          # packages dist/linux-unpacked (also dist:win, dist:mac)
```

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
