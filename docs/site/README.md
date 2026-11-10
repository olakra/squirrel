# Engineering docs site

The source for the GitHub Pages site. It renders the engineering standards in `docs/engineering/`, the Axiom
architecture in `projects/axiom/docs/`, and the diagrams page in `pages/diagrams.md`. The Markdown files stay the single
source. Nothing here copies their text by hand.

## Files

| Path | Purpose |
|---|---|
| `build.sh` | Entry point. Runs `build.py` with the right paths. Writes `_site/`. |
| `build.py` | Stages the sources, renders HTML, writes the output and runs the checks. Standard library only. |
| `_config.yml` | Site title, description, studio name and tagline, base path and the GitHub links. |
| `_layouts/default.html` | Page layout: pre-paint theme snippet, studio header, nav, theme toggle, content, footer. |
| `_includes/theme.html` | Theme toggle script (auto, light, dark). Saves the choice and tells Mermaid to re-render. |
| `_includes/mermaid.html` | Loads Mermaid from the CDN and renders each `language-mermaid` block in the site theme. |
| `assets/vendor/pico/` | Vendored Pico CSS 2.1.1 (classless build), its MIT `LICENSE` and a source note. Do not edit. |
| `assets/site.css` | Studio brand layer on top of Pico: tokens per theme, type, reading width, code, tables. |
| `index.md` | Landing page. |
| `pages/diagrams.md` | System context, container view, belt tick, inserter states and release pipeline. |
| `.gitignore` | Keeps `_site/` out of git. |

The workflow is `.github/workflows/pages.yml`. It runs `build.sh` and deploys with the Actions Pages flow.

## Why GitHub Actions, not the `/docs` folder

The repository's `/docs` folder holds engineering sources. The branch-based Pages source would publish those files as
raw Markdown and skip the layout. The Actions deployment publishes only what `build.sh` writes.

## Why no Jekyll build

Jekyll is the default Pages renderer, but this site does not use it, for two reasons.

1. A Jekyll build needs the `jekyll` or `github-pages` gems from RubyGems at build time. The build must run offline,
   and the Pages job would have to install gems first.
2. Jekyll is not installed in the build sandbox. A Jekyll build could not be checked here, so the site would have
   shipped unverified.

`build.py` uses only the Python standard library, so the build needs no network access. It reads `_config.yml`, the
layout and the includes, and it uses the same Jekyll-style front matter. A future move to Jekyll would need no change to
the sources, but that path is not tested.

## Build

```sh
bash docs/site/build.sh                              # writes docs/site/_site/
SITE_BASEURL=/squirrel bash docs/site/build.sh       # the project-page path that the workflow uses
```

`SITE_OUT` changes the output directory. `SITE_STAGE` keeps the staging tree for inspection. Without it, the staging
tree
goes to a temporary directory and is removed.

Staging and rendering run in these steps:

1. Each source file gets front matter (`title` and `layout`). Its title is its first `#` heading.
2. `{{ site.* }}` values come from `_config.yml`. An unknown key fails the build.
3. Links are resolved. A link to another page becomes a link to the staged page. A link to a repository file becomes a
   GitHub link on `main`. A link that resolves to nothing fails the build.
4. `README.md` in a folder becomes that folder's `index.html`.

## What the build checks

- Every staged page has `title` and `layout` in its front matter.
- Every internal link in the output points at a file that exists, and every `#anchor` matches an `id` on its target
  page.
- Every link that leaves the site resolves to a repository file or directory, or is a `http(s)` or `mailto` URL.
  External
  URLs are not fetched.
- Every Mermaid block starts with a known diagram keyword and has balanced brackets. This is a structure check only. It
  does not render the diagram, so a syntax error inside a diagram is not caught here.

The build exits with status 1 on any error. A broken link therefore fails the Pages workflow before it deploys.

## Supported Markdown

The renderer handles the subset these docs use: ATX headings with ids, paragraphs, nested ordered and unordered lists
(with fenced code inside items), block quotes, pipe tables, fenced code with a language, inline code, links, bold,
italic and horizontal rules. Raw HTML in the sources is escaped, not passed through. Anything outside this subset
renders as plain text, so check a new construct in the output before you rely on it.

## Theme

The base theme is [Pico CSS](https://picocss.com) 2.1.1, the classless build, under the MIT license. It styles plain
semantic HTML, so the renderer's output needs no classes, and it ships light and dark themes. The file is vendored in
`assets/vendor/pico/` so the build and the site need no network for styles. `build.py` copies all of `assets/`, so the
vendor folder ships with no build change.

### Brand layer

`assets/site.css` loads after Pico and holds only the studio look:

- Tokens named `--sq-*` (paper, bark, acorn amber) are set once per theme and mapped onto Pico's variables, such as
  `--pico-primary`, `--pico-background-color` and `--pico-code-background-color`. Change a colour in the token block,
  not in a rule.
- Every text and background pair meets WCAG AA (4.5:1) in both themes. Check a new colour before you commit it.
- Headings use a rounded system font stack (`ui-rounded`, then other rounded or geometric faces). Body text uses
  `system-ui`. No font is downloaded.
- The content column is 46 rem wide with a fixed 16 px gutter. Tables scroll inside their `.table-wrap` box on phones,
  so the page itself never scrolls sideways.

### Light, dark and auto

The theme follows Pico's convention on `<html>`:

| `data-theme` | Result |
|---|---|
| absent | Auto. Follows the OS `prefers-color-scheme` setting. |
| `light` | Light, whatever the OS says. |
| `dark` | Dark, whatever the OS says. |

The header button cycles auto, light, dark. It saves the choice in `localStorage` under `squirrel-theme`, inside
`try`/`catch`, so a blocked store only means the choice is not remembered. A small script in `<head>` applies the
saved value before the first paint, so a forced theme does not flash. On each change the toggle fires a
`squirrel:themechange` event. The Mermaid include listens for it and re-renders each diagram from its saved source
with the `dark` or `default` Mermaid theme. Two `theme-color` meta tags colour the browser bar for each OS scheme.

### Upgrading Pico

1. Pick an exact 2.x version, at least two weeks old, from <https://www.npmjs.com/package/@picocss/pico>.
2. Download `css/pico.classless.min.css` and `LICENSE.md` for that version from
   `https://cdn.jsdelivr.net/npm/@picocss/pico@<version>/`. Save them as `assets/vendor/pico/pico.classless.min.css`
   and `assets/vendor/pico/LICENSE`.
3. Update the version and the SHA-256 in `assets/vendor/pico/README.md` and in this section.
4. Build, then check the header, a table, a code block and a diagram in light and dark. A major version (3.x) may
   rename the `--pico-*` variables that `site.css` maps.

## Mermaid

The include loads `https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.min.js` at view time. It finds every
`pre > code.language-mermaid` block, turns it into a diagram and renders it. When the script does not load, the Mermaid
source stays on the page.

**Subresource Integrity is not pinned.** The URL names only the major version, so the file changes with every 11.x
release. A hash computed today would fail after the next release, and the hash could not be computed here anyway,
because
the build sandbox cannot reach the CDN. Pinning is a follow-up: choose an exact version, download that file, compute its
`sha384` hash, and add `integrity` and `crossorigin` attributes to the script tag.

## Verification status

Checked in the build sandbox:

- `build.sh` runs and writes 16 pages. Every internal link and anchor resolves, with the root and with `/squirrel`.
- A negative test with a broken link, a missing anchor and a bad Mermaid keyword makes the build exit 1 and names each
  error.
- Every `.yml` file under `.github/` and `docs/site/_config.yml` parses with `yaml.safe_load`.
- `bash -n` passes on `docs/site/build.sh` and on `scripts/ci/*.sh`.

Not checked:

- A Jekyll build. Jekyll is not used (see above).
- Mermaid rendering in a browser. The CDN is not reachable from the build sandbox. The five diagrams were checked
  against
  the source code they describe, and their syntax was checked for a known keyword and balanced brackets only.
- `shellcheck`, which is not installed here.
- `markdownlint-cli2`, which needs `npx` and therefore the network.

## Owner steps

1. Set the source once: GitHub, Settings, Pages, Build and deployment, Source, **GitHub Actions**. This is a one-time
   change. The API call that does it was refused from the build sandbox.
2. Merge the change to `main`. The `Pages` workflow runs on pushes that touch `docs/**` or `projects/axiom/docs/**`.
3. Check the first run in the Actions tab, then open the Pages URL shown in the `deploy` job.
4. `docs/roadmap.md` does not exist yet. The Roadmap link returns 404 until its owner adds the file on `main`.
5. Pin Mermaid to an exact version with an SRI hash (see above) when the first release of this site is ready.
