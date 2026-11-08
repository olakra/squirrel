# ADR 0001: Electron as the desktop shell

- **Status:** Proposed
- **Date:** 2026-11-08
- **Deciders:** repository owner (@olakra)

## Context

Axiom must ship on Steam for Windows, macOS, Linux and the Steam Deck (SteamOS, Linux with Proton and
a gamepad-first UI). The browser build in `projects/axiom/web` must run offline in a native window. It
needs three things from the shell:

- A Gamepad API and a window that accepts full controller input.
- Cross-origin isolation, so `SharedArrayBuffer` and the worker can run.
- Module workers and `fetch()` of `axiom.wasm`, which do not work from `file://`.

Two shells were considered. Both load `web/` from a bundled folder and both need no network at runtime.

- **Tauri 2.** Rust-native, small. It uses each OS's system web view: WebView2 on Windows, WKWebView on
  macOS and WebKitGTK on Linux. The task asked for it as the preferred option.
- **Electron.** Chromium is bundled with every release, so the engine is the same on all three OSes.
  Larger output, and a Node.js build toolchain.

The decisive factor is the Linux and Steam Deck target:

1. **The Linux engine is a system library.** Tauri on Linux needs `libwebkit2gtk-4.1` at run time.
   We could not confirm that SteamOS ships it by default, or that the Steam Linux Runtime provides it. We
   would have to bundle WebKitGTK into the depot ourselves, or verify the runtime provides it.
2. **The Gamepad API depends on the WebKitGTK build.** Its feature set and version differ across distros.
   On the Deck, controller input is the main requirement, so the one platform we cannot pin is the one
   that matters most.
3. **Electron pins its engine per release.** The same Chromium version runs on every OS, so a Steam Deck
   test result carries over to Windows and macOS.

Costs we accept with Electron: a larger download per platform (Chromium is bundled), a Node toolchain
in CI, and a Linux sandbox that needs checking on the Deck (see Consequences).

Tauri remains a valid choice for a later, smaller release. Revisit it if the bundle size becomes a
problem and if WebKitGTK is verified on SteamOS.

## Decision

We will use Electron, in `projects/axiom/desktop`, with these choices:

- The app loads `web/` through a privileged `axiom://app` scheme, not `file://`. Responses carry COOP,
  COEP and CORP headers and a Content-Security-Policy.
- The renderer runs sandboxed with context isolation. Its preload exposes a read-only description and no IPC.
- The network is blocked except for the app's own origin.
- Output is unpacked folders (`electron-builder --dir`). Steam depots take folders.
- The shell contains no Steam code. The Steamworks SDK goes in a separate module later.

## Consequences

- **Easier:** one engine on every OS, so one test pass covers the Deck and the desktop. Gamepad behaviour
  is the same everywhere. Steam depots are plain folders.
- **Harder:** the output is large. The Node toolchain is a dependency of CI. Each Electron major needs an
  upgrade within its support window.
- **Linux sandbox risk, not yet verified on a Deck.** Chromium's sandbox needs either unprivileged user
  namespaces or a setuid `chrome-sandbox` helper. A Steam depot cannot set the setuid bit. If the Deck
  build aborts with a sandbox error, the documented fallback is `--no-sandbox` in the Linux launch
  options. Enable it only after that test fails, and record the reason. See `desktop/README.md`.
- **Not built here.** The network blocks package installs, so `npm ci` and `electron-builder` have not been
  run. `package-lock.json` does not exist yet. Commit it after the first `npm install` on a networked machine.
- **Follow-up:** after a preview build runs on a Deck, record the result in this ADR and set Status to Accepted.
