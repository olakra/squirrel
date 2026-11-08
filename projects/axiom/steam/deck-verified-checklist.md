# Steam Deck Verified checklist

Use this before submitting Axiom for Steam Deck Verified review. Valve's own criteria and test
flow are authoritative. Where this list is unsure, the item says "verify in Steamworks docs".
Re-check the criteria on the day you submit, because Valve revises them.

Status key:

- **Done in code**: implemented in this repository. It still needs a check on a real Deck.
- **Owner test**: only someone with a Deck can confirm it.
- **Verify**: confirm the exact rule in the Steamworks documentation.

## Controls

| # | Requirement | Status |
|---|---|---|
| 1 | Every action can be done with the controller alone. No mouse or keyboard is required. | Done in code: cursor by stick or D-pad, A confirm, B cancel, Start hints. Owner test: complete a session with no keyboard. |
| 2 | On-screen prompts show controller glyphs, not keyboard keys, when a controller is in use. | Done in code: the hint footer switches to controller labels after a controller input. Owner test on the Deck. The labels are plain text chips, not Steam's glyph art (verify whether Steam requires its art). |
| 3 | Controls work with Steam Input's default configuration. | Verify in Steamworks docs. The web layer uses the W3C standard mapping (Xbox layout), which Steam Input presents. |
| 4 | Any text entry works through Steam's on-screen keyboard. | Not applicable today: the game has no text entry. Re-check when it does. |
| 5 | A controller must be detected after the player presses a button. | Owner test. Chromium and Electron expose gamepads after a button press. The hint footer shows keyboard hints until then. |

## Display and text

| # | Requirement | Status |
|---|---|---|
| 6 | Text is readable at 1280x800 without zooming. | Done in code: `web/styles/deck.css` sets body and HUD text to at least 18 px, and 20 px at 1280x800. Verify the exact minimum in the Steamworks docs. |
| 7 | The game runs at the Deck's native 1280x800 with correct scaling. | Done in code: the desktop window opens at 1280x800 and resizes. Owner test: check scaling in Gaming Mode. |
| 8 | Fullscreen works. | Done in code: F11 or Alt+Enter in the desktop shell. Gaming Mode is already fullscreen. Owner test. |
| 9 | Focus indicators are visible on the Deck screen. | Done in code: `:focus-visible` outline and a high-contrast cursor. Owner test on the Deck screen. |

## Launch and platform

| # | Requirement | Status |
|---|---|---|
| 10 | The game starts from Gaming Mode with no mouse, no keyboard and no desktop-mode step. | Owner test. |
| 11 | The Linux native build starts on SteamOS. | Owner test. Electron on Linux needs either a working user-namespace sandbox or the chrome-sandbox helper. Steam's runtime may change which works. If it fails, see `desktop/README.md` before you change any launch option. |
| 12 | Launch options cover every OS. | Done in the reference file `launch-options.template.json`. Enter the values in Steamworks by hand. |
| 13 | The Proton (Windows) build also runs on the Deck, if you ship one. | Owner test. Verify in Steamworks docs whether Valve reviews the Linux or Windows build for Deck Verified. |
| 14 | The game works after suspend and resume. | Owner test. |
| 15 | No launcher or external browser window needs a mouse. | Done in code: the desktop shell blocks navigation and new windows. The network is blocked too, so no external page can open. |

## Stability and content

| # | Requirement | Status |
|---|---|---|
| 16 | The game does not crash in normal play on the Deck. | Owner test. |
| 17 | The frame rate is playable on the Deck. | Owner test. Verify the target in the Steamworks docs. The renderer is still a placeholder. |
| 18 | The game runs with no internet connection. | Done in code: the desktop shell allows only the bundled app origin. |
| 19 | Save data is stored in a standard place and survives a restart. | Not yet applicable. There is no save system. Re-check when saves exist. |
| 20 | The Steamworks "Steam Deck compatibility" test flow passes. | Owner test. Verify the current steps in the Steamworks docs. |

## Before you submit

- [ ] Every row above is marked done, or you have a reason to skip it.
- [ ] You tested a preview build on a real Deck in Gaming Mode.
- [ ] You checked the current Steam Deck Verified criteria in the Steamworks docs, because they change.
