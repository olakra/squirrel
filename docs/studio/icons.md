# Icons

This document lists every icon the game needs and the rules each icon follows. Icons are drawn
by the UI/UX Designer and exported as SVG. Colours come from the palette in `art-direction.md`.

## Icon set

| Icon | File | Meaning | Where it is used | Colour token |
|---|---|---|---|---|
| New game | `new-game.svg` | Start a fresh round | Result panel and header | `--color-text` |
| Reset | `reset.svg` | Clear the board and scores | Header | `--color-text` |
| X mark | `mark-x.svg` | Player X's move | Board cells | `--color-x` |
| O mark | `mark-o.svg` | Player O's move | Board cells | `--color-o` |
| Win highlight | `win-line.svg` | Line through the three winning cells | Board overlay | `--color-win` |
| Draw | `draw.svg` | A full board with no winner | Result panel | `--color-text` |
| Sound on | `sound-on.svg` | Sound is enabled | Header toggle | `--color-text` |
| Sound off | `sound-off.svg` | Sound is muted | Header toggle | `--color-text` |
| Language switch | `language.svg` | Opens the language menu | Header | `--color-text` |
| Favicon | `favicon.svg` | Browser tab icon | Browser tab | `--color-x` and `--color-o` |
| App icon 192 | `app-icon-192.png` | Home screen icon, small | Web app manifest | Exported from `app-icon.svg` |
| App icon 512 | `app-icon-512.png` | Home screen icon, large | Web app manifest | Exported from `app-icon.svg` |

Notes on the set:

- **Win highlight** is a line, not a glyph. It is drawn over the board at a 4px stroke with
  rounded caps. Its stroke is not governed by the 2px icon rule, because it is part of the board.
- **Draw** is a pair of short horizontal bars, so it reads as "even" and not as an error.
- **Sound on and sound off** are two files, not one icon with a slash added in code. Each is
  drawn in full on the grid.
- **Language switch** shows an icon only. The visible text label, such as the current language
  name, is HTML text, so it can be translated.
- **Favicon and app icons** are the only icons that are not drawn at 24px. They use the rules
  for scaled artwork below.

## Grid rules

These rules apply to every icon in the table except the favicon and the app icons, which
follow the scaled-artwork rule.

- **Artboard.** 24 by 24 px, with a live area of 20 by 20 px. Keep all artwork inside the live
  area so no edge is clipped at small sizes.
- **Pixel grid.** Align straight edges and stroke centres to the 24px grid, so lines are crisp
  at 1x and 2x.
- **Stroke.** 2px, centred on the path. No filled shapes unless the icon is a solid mark such as
  the X and O marks, which are stroked, not filled.
- **Caps and joins.** Round caps and round joins on every path.
- **Corners.** Where a shape has a corner, use a 2px radius on the centreline, so the inner
  corner stays clean.
- **Colour.** One colour only, taken from a palette token. Use `currentColor` where the icon
  inherits from text, and set the token on the parent. Do not use gradients, shadows or
  opacity to make a second tone.
- **Size in use.** Icons render at 24px in the UI. A larger icon in a touch target is drawn at
  24px and the hit area is extended to 44 by 44 CSS px, as set in `art-direction.md`.
- **Naming.** Lowercase, words joined with hyphens, and the file name describes the meaning,
  not the look. Use `sound-off.svg`, not `speaker-slash.svg`.

## Scaled artwork

The favicon and the app icons are drawn on their own artboard, then scaled from it.

- The app icon artboard is 512 by 512 px. The 24px icon is scaled up so the stroke keeps its
  ratio: a 2 in 24 stroke becomes a 42.67px stroke on the 512 artboard.
- The 192 icon is the same artwork exported at 192 px. No separate design.
- The favicon is a simplified mark with the X and O only. It is exported at 16 px and 32 px.
  At 16 px the stroke must not drop below 2px, so the favicon may use a heavier stroke than the
  ratio rule gives.
- All app and favicon artwork uses the background token `--color-bg` as a solid square fill,
  so the icon has no transparent edges on some platforms.

## Accessibility

- An icon that stands alone on a button needs an accessible name. Put the name in the button's
  `aria-label`, and translate it.
- An icon that sits next to visible text is decorative. Set `aria-hidden="true"` on it.
- Toggles such as sound on and sound off use `aria-pressed` on the button. The icon swaps
  between the two files, and the label stays the same.
- Icons never carry words inside the image. See the i18n rules in `art-direction.md`.
