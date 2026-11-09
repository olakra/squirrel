# Art direction

## Design philosophy

The game is modern and minimalist. It has one job: a player reads the board, makes a move and
sees the result. Every element that does not help with that job is removed.

- **Few colours.** One neutral family for the page and text, one for the grid, and two marks.
  A single green is used only for a win.
- **Shape carries meaning.** X and O are distinct shapes. A player must be able to read the board
  in greyscale.
- **Space is a feature.** Generous padding and clear gaps. The board is the largest thing on
  the screen.
- **Motion explains, it does not decorate.** Motion is used only to show a change of state, and
  it is short.
- **Accessible by default.** Contrast, focus, touch size and motion rules are part of the design,
  not checks added at the end.

## Colour palette

All colours are defined as CSS custom properties on `:root`. Components use the token name, never
the hex value. Each token lists the background it was measured against. Ratios are computed with
the WCAG 2.x relative luminance formula, which WCAG 2.2 still uses.

Text tokens must reach 4.5:1 against their background. Graphic tokens, such as grid lines, marks,
the focus ring and the win highlight, must reach 3:1 against the surface they sit on.

| Token | Hex | Use | Measured against | Ratio |
|---|---|---|---|---|
| `--color-bg` | `#FAFAF9` | Page background | `--color-text` on it | 16.47:1 |
| `--color-surface` | `#FFFFFF` | Board and panels | `--color-text` on it | 17.20:1 |
| `--color-surface-hover` | `#F1F0EE` | Cell under the pointer | `--color-text` on it | 15.10:1 |
| `--color-text` | `#1C1B1A` | Primary text | `--color-bg` | 16.47:1 |
| `--color-text-muted` | `#5C5A56` | Secondary text and hints | `--color-bg` | 6.59:1 |
| `--color-line` | `#767370` | Grid lines and cell borders | `--color-surface` | 4.71:1 |
| `--color-x` | `#1D4ED8` | The X mark | `--color-surface` | 6.70:1 |
| `--color-o` | `#A34A07` | The O mark | `--color-surface` | 5.93:1 |
| `--color-win` | `#15803D` | Winning line highlight | `--color-surface` | 5.02:1 |
| `--color-focus` | `#1D4ED8` | Focus ring | `--color-bg` | 6.42:1 |
| `--color-action-bg` | `#1C1B1A` | Primary button fill | `--color-action-text` on it | 17.20:1 |
| `--color-action-text` | `#FFFFFF` | Primary button label | `--color-action-bg` | 17.20:1 |

Notes on the table:

- `--color-line` is the darkest grid tone that still stays quiet. It passes 3:1 on the surface.
  It is 4.14:1 on `--color-surface-hover`, so the grid stays visible under the pointer.
- `--color-x` and `--color-focus` share a hex value on purpose. Both are blue, and the focus ring
  is never drawn on an X mark, so the two never compete.
- `--color-o` is warm and `--color-x` is cool, so the two marks differ in hue as well as shape.
- `--color-text-muted` is for hints only. Do not use it for anything a player must read to
  finish a move.

The same palette as CSS:

```css
:root {
  --color-bg: #FAFAF9;
  --color-surface: #FFFFFF;
  --color-surface-hover: #F1F0EE;
  --color-text: #1C1B1A;
  --color-text-muted: #5C5A56;
  --color-line: #767370;
  --color-x: #1D4ED8;
  --color-o: #A34A07;
  --color-win: #15803D;
  --color-focus: #1D4ED8;
  --color-action-bg: #1C1B1A;
  --color-action-text: #FFFFFF;
}
```

Colour is never the only cue. A win is shown by a line and by a status message. A mark is
shown by its shape.

## Type scale

The base size is 16px. Sizes use a 1.25 ratio, rounded to whole pixels. The UI font is the system
UI stack. Do not load a web font for the first version.

| Token | Size | Line height | Use |
|---|---|---|---|
| `--font-size-sm` | 14px | 1.5 | Hints and footnotes |
| `--font-size-base` | 16px | 1.5 | Body text and buttons |
| `--font-size-lg` | 20px | 1.4 | Status message |
| `--font-size-xl` | 24px | 1.3 | Screen headings |
| `--font-size-display` | 32px | 1.2 | Result of a round |

Font weights: 400 for body, 600 for buttons and headings. The values above are the design
reference in pixels. In CSS, write each size in `rem` (16px is `1rem`), so browser text size
settings still apply.

## Spacing scale

The base unit is 4px. Every margin, padding and gap uses one of these values. Do not use other
values without a reason written in the pull request.

| Token | Value |
|---|---|
| `--space-1` | 4px |
| `--space-2` | 8px |
| `--space-3` | 12px |
| `--space-4` | 16px |
| `--space-6` | 24px |
| `--space-8` | 32px |
| `--space-12` | 48px |
| `--space-16` | 64px |

## Corner radius

| Token | Value | Use |
|---|---|---|
| `--radius-sm` | 4px | Focus ring corners and small chips |
| `--radius-md` | 8px | Cells, buttons and panels |
| `--radius-lg` | 16px | The board container |

## Touch targets

Every interactive element is at least 44 by 44 CSS pixels. This applies to each board cell, each
button, each icon button and each language or sound toggle. If the visible shape is smaller, the
hit area is extended with padding or a pseudo-element, so the visual size can stay small.

## Focus ring

Every focusable element shows a visible focus ring when it receives keyboard focus.

- Style: `outline: 3px solid var(--color-focus); outline-offset: 2px;`
- Use `:focus-visible`, so the ring shows for keyboard users and not for mouse clicks.
- Never set `outline: none` without a replacement that meets the same contrast rule.
- The ring must have at least 3:1 contrast against the surface it sits on. The focus token does
  this against both `--color-bg` and `--color-surface`.

## Motion

- Use two durations only: `120ms` for small state changes such as hover, and `200ms` for the
  appearance of a mark or the win line.
- Easing for all motion: `cubic-bezier(0.2, 0, 0, 1)`.
- Animate `opacity`, `transform` and `stroke-dashoffset` only. Do not animate layout properties.
- A mark appears with a short opacity fade. The win line is drawn once, from start to end.
- Respect reduced motion. Wrap every animation in a `prefers-reduced-motion` check:

```css
@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
    scroll-behavior: auto !important;
  }
}
```

With reduced motion, a mark appears at once and the win line is shown without drawing. The
information is the same, only the movement is removed.

## Internationalisation rules

- **Text expansion.** Every text container must hold 30 percent more text than the English string
  without clipping, overlap or a horizontal scroll. Test with a string 1.3 times the English length.
- **No text in images.** Every word the player reads is HTML text, including labels inside icons.
  Images and icons carry no words. Exceptions need the Localisation Lead's approval.
- **Logical properties.** Use `margin-inline-start`, `padding-inline-end`, `inset-inline-start`,
  `border-start-start-radius` and similar. Do not use `left`, `right`, `margin-left` or
  `padding-right` in new CSS.
- **Right-to-left.** The page sets `dir` from the chosen language. Layout flows by logical
  direction, so it mirrors without extra rules.
- **Board stays fixed.** The 3 by 3 grid is a spatial game layout, so it does not mirror.
  Top-left is always the first cell. Directional icons, such as a back arrow, do mirror.
- **Language in markup.** Set `lang` on the root element to the code of the chosen language.
- **Numbers and dates.** Use `Intl` formatters. Do not build numbers or dates by hand.
