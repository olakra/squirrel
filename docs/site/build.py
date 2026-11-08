#!/usr/bin/env python3
"""Build the squirrel engineering docs into a static site.

Standard library only. Nothing is downloaded at build time. See docs/site/README.md.

Pipeline:
  1. Stage. Copy the Markdown sources into a staging tree, add front matter, fill in
     {{ site.* }} values, and rewrite links so they resolve inside the site or point to GitHub.
  2. Render. Turn each staged page into HTML with _layouts/default.html and _includes/.
  3. Write, then check. Every internal link and anchor in the output must resolve, every staged
     page must have front matter, and every Mermaid block must start with a known diagram keyword
     and have balanced brackets.

The exit status is 1 on any error, so a broken link fails the Pages build.

Usage: build.py --repo-root DIR --site-dir DIR --stage DIR --out DIR
Environment: SITE_BASEURL overrides the baseurl in _config.yml (for example "/squirrel").
"""

import argparse
import html
import html.parser
import json
import os
import posixpath
import re
import shutil
import sys

# The first word of every Mermaid block must be one of these diagram keywords.
MERMAID_KEYWORDS = (
    "flowchart",
    "graph",
    "sequenceDiagram",
    "stateDiagram-v2",
    "stateDiagram",
    "classDiagram",
    "erDiagram",
    "gantt",
    "pie",
    "journey",
    "gitGraph",
    "mindmap",
    "timeline",
    "quadrantChart",
    "requirementDiagram",
    "C4Context",
    "sankey-beta",
    "xychart-beta",
    "block-beta",
)

EXTERNAL_RE = re.compile(r"^(?:[a-zA-Z][a-zA-Z0-9+.-]*:|//)")
LINK_RE = re.compile(r"\[([^\]]*)\]\(([^)\s]+)\)")
FENCE_OPEN_RE = re.compile(r"^(\s*)(`{3,}|~{3,})\s*([^\s`]*)")
SITE_VAR_RE = re.compile(r"\{\{\s*site\.(\w+)\s*\}\}")
PAGE_VAR_RE = re.compile(r"\{\{\s*page\.(\w+)\s*\}\}")
INCLUDE_RE = re.compile(r"\{%\s*include\s+([\w.-]+)\s*%\}")
HEADING_RE = re.compile(r"^\s{0,3}(#{1,6})\s+(.+?)\s*$")
HR_RE = re.compile(r"^\s{0,3}([-*_])(?:\s*\1){2,}\s*$")
QUOTE_RE = re.compile(r"^\s{0,3}>\s?(.*)$")
UL_RE = re.compile(r"^(\s*)([-*+])\s+(.*)$")
OL_RE = re.compile(r"^(\s*)(\d{1,9})[.)]\s+(.*)$")
TABLE_SEP_RE = re.compile(r"^\s*\|?\s*:?-+:?\s*(?:\|\s*:?-+:?\s*)*\|?\s*$")
CODE_SPAN_RE = re.compile(r"(`[^`]+`)")
BOLD_RE = re.compile(r"\*\*(?=\S)(.+?)(?<=\S)\*\*")
ITALIC_RE = re.compile(r"(?<![\w*])\*(?=\S)(.+?)(?<=\S)\*(?![\w*])")

GITHUB_BRANCH = "main"


# ---------------------------------------------------------------- small helpers


def normalize_base(value):
    """Turns a configured base path into "" or "/segment" with no trailing slash."""
    value = (value or "").strip()
    if not value:
        return ""
    return "/" + value.strip("/")


def read_config(path):
    """Reads flat `key: value` pairs from _config.yml. Only top-level scalars are supported."""
    values = {}
    with open(path, encoding="utf-8") as handle:
        for raw in handle:
            line = raw.rstrip("\n")
            if not line.strip() or line.lstrip().startswith("#") or line[:1].isspace():
                continue
            key, sep, value = line.partition(":")
            if not sep:
                continue
            value = value.split(" #", 1)[0].strip()
            values[key.strip()] = unquote(value)
    return values


def unquote(value):
    if len(value) >= 2 and value[0] == value[-1] and value[0] in "\"'":
        return value[1:-1]
    return value


def split_front_matter(text):
    """Returns (meta, body). Only flat `key: value` lines are read."""
    if not text.startswith("---\n"):
        return {}, text
    end = text.find("\n---\n", 4)
    if end == -1:
        raise ValueError("front matter is not closed")
    meta = {}
    for line in text[4:end].split("\n"):
        key, sep, value = line.partition(":")
        if sep and key.strip():
            meta[key.strip()] = unquote(value.strip())
    return meta, text[end + 5 :]


def dump_front_matter(meta):
    lines = ["---"]
    for key, value in meta.items():
        lines.append(f"{key}: {json.dumps(str(value), ensure_ascii=False)}")
    lines.append("---")
    return "\n".join(lines) + "\n\n"


def plain_text(text):
    """Strips the inline Markdown that would show up in a title or a heading slug."""
    text = LINK_RE.sub(lambda m: m.group(1), text)
    return text.replace("`", "").replace("**", "").replace("*", "").strip()


def first_heading(body):
    match = re.search(r"^#\s+(.+?)\s*$", body, re.M)
    return plain_text(match.group(1)) if match else ""


def slugify(text):
    text = re.sub(r"[^a-z0-9 _-]", "", text.lower())
    return re.sub(r"\s+", "-", text.strip()) or "section"


def indent_of(line):
    return len(line) - len(line.lstrip(" "))


def dedent_upto(line, width):
    return line[min(indent_of(line), width) :]


def list_item(line):
    """Returns (kind, indent, number, content, prefix_width) for a list line, else None."""
    match = UL_RE.match(line)
    if match:
        return ("ul", len(match.group(1)), None, match.group(3), len(line) - len(match.group(3)))
    match = OL_RE.match(line)
    if match:
        return ("ol", len(match.group(1)), int(match.group(2)), match.group(3), len(line) - len(match.group(3)))
    return None


def split_row(line):
    """Splits a pipe-table row into cells. Pipes inside code spans and escaped pipes are kept."""
    text = line.strip()
    if text.startswith("|"):
        text = text[1:]
    if text.endswith("|") and not text.endswith("\\|"):
        text = text[:-1]
    cells, current, in_code = [], "", False
    for ch in text:
        if ch == "`":
            in_code = not in_code
        if ch == "|" and not in_code:
            if current.endswith("\\"):
                current = current[:-1] + "|"
                continue
            cells.append(current.strip())
            current = ""
        else:
            current += ch
    cells.append(current.strip())
    return cells


def check_mermaid(code):
    """Returns an error string for a structurally broken Mermaid block, or None."""
    lines = [ln.strip() for ln in code.splitlines() if ln.strip() and not ln.strip().startswith("%%")]
    if not lines:
        return "empty Mermaid block"
    first = lines[0].split()[0]
    if first not in MERMAID_KEYWORDS:
        return f"Mermaid block starts with unknown keyword '{first}'"
    stripped = re.sub(r'"[^"]*"', "", code)
    pairs = {")": "(", "]": "[", "}": "{"}
    stack = []
    for ch in stripped:
        if ch in "([{":
            stack.append(ch)
        elif ch in pairs:
            if not stack or stack.pop() != pairs[ch]:
                return "Mermaid block has unbalanced brackets"
    if stack:
        return "Mermaid block has unbalanced brackets"
    return None


# ---------------------------------------------------------------- Markdown renderer


class Markdown:
    """Renders the Markdown subset these docs use: headings, paragraphs, nested lists, block
    quotes, pipe tables, fenced code, inline code, links, bold, italic and horizontal rules."""

    def __init__(self, href):
        self.href = href  # callable: link target in Markdown -> href in HTML
        self.ids = set()
        self.mermaid = []

    def render(self, text):
        return self.blocks(text.expandtabs(4).split("\n"))

    def starts_block(self, lines, i):
        line = lines[i]
        if FENCE_OPEN_RE.match(line) or HEADING_RE.match(line) or HR_RE.match(line) or QUOTE_RE.match(line):
            return True
        if list_item(line):
            return True
        return self.is_table_start(lines, i)

    @staticmethod
    def is_table_start(lines, i):
        return (
            "|" in lines[i]
            and i + 1 < len(lines)
            and "|" in lines[i + 1]
            and bool(TABLE_SEP_RE.match(lines[i + 1]))
        )

    def blocks(self, lines):
        out = []
        i, n = 0, len(lines)
        while i < n:
            line = lines[i]
            if not line.strip():
                i += 1
                continue
            if FENCE_OPEN_RE.match(line):
                i = self.fenced(lines, i, out)
                continue
            heading = HEADING_RE.match(line)
            if heading:
                level = len(heading.group(1))
                text = re.sub(r"\s+#+$", "", heading.group(2))
                slug = self.unique_slug(slugify(plain_text(text)))
                out.append(f'<h{level} id="{slug}">{self.inline(text)}</h{level}>')
                i += 1
                continue
            if HR_RE.match(line):
                out.append("<hr>")
                i += 1
                continue
            if QUOTE_RE.match(line):
                inner = []
                while i < n and QUOTE_RE.match(lines[i]):
                    inner.append(QUOTE_RE.match(lines[i]).group(1))
                    i += 1
                out.append("<blockquote>\n" + self.blocks(inner) + "</blockquote>")
                continue
            if self.is_table_start(lines, i):
                i = self.table(lines, i, out)
                continue
            if list_item(line):
                i = self.list_block(lines, i, out)
                continue
            para = [line.strip()]
            i += 1
            while i < n and lines[i].strip() and not self.starts_block(lines, i):
                para.append(lines[i].strip())
                i += 1
            out.append("<p>" + self.inline(" ".join(para)) + "</p>")
        return "\n".join(out) + ("\n" if out else "")

    def unique_slug(self, slug):
        candidate, suffix = slug, 1
        while candidate in self.ids:
            candidate = f"{slug}-{suffix}"
            suffix += 1
        self.ids.add(candidate)
        return candidate

    def fenced(self, lines, i, out):
        match = FENCE_OPEN_RE.match(lines[i])
        indent, fence, lang = len(match.group(1)), match.group(2), match.group(3)
        i += 1
        body = []
        while i < len(lines):
            stripped = lines[i].strip()
            if stripped and set(stripped) == {fence[0]} and len(stripped) >= len(fence):
                break
            body.append(dedent_upto(lines[i], indent))
            i += 1
        i += 1  # the closing fence, or the end of the input
        code = "\n".join(body) + "\n" if body else ""
        if lang == "mermaid":
            self.mermaid.append(code)
        cls = f' class="language-{html.escape(lang, quote=True)}"' if lang else ""
        out.append(f"<pre><code{cls}>{html.escape(code, quote=False)}</code></pre>")
        return i

    def table(self, lines, i, out):
        header = split_row(lines[i])
        seps = split_row(lines[i + 1])
        aligns = []
        for sep in seps:
            if sep.startswith(":") and sep.endswith(":"):
                aligns.append("center")
            elif sep.endswith(":"):
                aligns.append("right")
            elif sep.startswith(":"):
                aligns.append("left")
            else:
                aligns.append("")
        i += 2
        rows = []
        while i < len(lines) and lines[i].strip() and "|" in lines[i]:
            rows.append(split_row(lines[i]))
            i += 1

        def cells(values, tag):
            pieces = []
            for k, value in enumerate(values):
                align = aligns[k] if k < len(aligns) else ""
                style = f' style="text-align:{align}"' if align else ""
                pieces.append(f"<{tag}{style}>{self.inline(value)}</{tag}>")
            return "".join(pieces)

        parts = ['<div class="table-wrap"><table>', "<thead><tr>" + cells(header, "th") + "</tr></thead>"]
        parts.append("<tbody>")
        for row in rows:
            padded = row + [""] * (len(header) - len(row))
            parts.append("<tr>" + cells(padded[: len(header)], "td") + "</tr>")
        parts.append("</tbody></table></div>")
        out.append("".join(parts))
        return i

    def list_block(self, lines, i, out):
        kind, base, start, _, _ = list_item(lines[i])
        items = []
        n = len(lines)
        while i < n:
            item = list_item(lines[i])
            if item is None or item[0] != kind or item[1] != base:
                break
            content, prefix = item[3], item[4]
            body = [content]
            i += 1
            while i < n:
                line = lines[i]
                if not line.strip():
                    j = i
                    while j < n and not lines[j].strip():
                        j += 1
                    if j < n and indent_of(lines[j]) >= prefix:
                        body.extend([""] * (j - i))
                        i = j
                        continue
                    break
                indent = indent_of(line)
                if indent >= prefix:
                    body.append(line[prefix:])
                    i += 1
                    continue
                if list_item(line) and indent > base:
                    body.append(line.strip())
                    i += 1
                    continue
                if self.starts_block(lines, i):
                    break
                body.append(line.strip())  # lazy continuation of the item's paragraph
                i += 1
            items.append(body)
            j = i
            while j < n and not lines[j].strip():
                j += 1
            nxt = list_item(lines[j]) if j < n else None
            if nxt and nxt[0] == kind and nxt[1] == base:
                i = j
            else:
                break
        tag = "ol" if kind == "ol" else "ul"
        attr = f' start="{start}"' if kind == "ol" and start not in (None, 1) else ""
        parts = [f"<{tag}{attr}>"]
        for body in items:
            parts.append("<li>" + self.blocks(body) + "</li>")
        parts.append(f"</{tag}>")
        out.append("\n".join(parts))
        return i

    def inline(self, text):
        pieces = []
        for k, part in enumerate(CODE_SPAN_RE.split(text)):
            if k % 2 == 1:
                pieces.append("<code>" + html.escape(part[1:-1], quote=False) + "</code>")
            else:
                pieces.append(self.inline_marks(part))
        return "".join(pieces)

    def inline_marks(self, raw):
        pieces, pos = [], 0
        for match in LINK_RE.finditer(raw):
            pieces.append(self.emphasis(html.escape(raw[pos : match.start()], quote=False)))
            label = self.emphasis(html.escape(match.group(1), quote=False))
            target = html.escape(self.href(match.group(2)), quote=True)
            pieces.append(f'<a href="{target}">{label}</a>')
            pos = match.end()
        pieces.append(self.emphasis(html.escape(raw[pos:], quote=False)))
        return "".join(pieces)

    @staticmethod
    def emphasis(escaped):
        escaped = BOLD_RE.sub(r"<strong>\1</strong>", escaped)
        return ITALIC_RE.sub(r"<em>\1</em>", escaped)


# ---------------------------------------------------------------- the site build


class LinkError(Exception):
    pass


class PageScan(html.parser.HTMLParser):
    """Collects the ids and the link targets of one rendered page."""

    def __init__(self):
        super().__init__()
        self.ids = set()
        self.targets = []

    def handle_starttag(self, tag, attrs):
        values = dict(attrs)
        if values.get("id"):
            self.ids.add(values["id"])
        if tag in ("a", "link") and values.get("href"):
            self.targets.append(values["href"])
        if tag == "script" and values.get("src"):
            self.targets.append(values["src"])


class Site:
    def __init__(self, repo, site_dir, stage, out):
        self.repo = os.path.abspath(repo)
        self.site_dir = os.path.abspath(site_dir)
        self.stage_dir = os.path.abspath(stage)
        self.out = os.path.abspath(out)
        self.config = read_config(os.path.join(self.site_dir, "_config.yml"))
        override = os.environ.get("SITE_BASEURL")
        if override is not None:
            self.config["baseurl"] = override
        self.config["baseurl"] = normalize_base(self.config.get("baseurl", ""))
        self.errors = []
        self.sources = {}  # repo-relative source path -> staged path
        self.staged = {}  # staged path -> repo-relative source path, or None when generated
        self.pages = {}  # staged path -> (front matter dict, Markdown body)
        self.html_pages = {}  # output path -> HTML text
        self.mermaid_blocks = []  # (staged path, code)
        self.external_links = 0

    # --- error reporting

    def error(self, where, message):
        self.errors.append(f"{where}: {message}")

    def site_value(self, key, where):
        if key not in self.config:
            self.error(where, f"unknown site value '{key}'")
            return ""
        return self.config[key]

    # --- step 1: collect and stage

    def repo_rel(self, path):
        return posixpath.normpath(os.path.relpath(path, self.repo).replace(os.sep, "/"))

    def collect(self):
        def add(repo_rel, staged_rel):
            self.staged[staged_rel] = repo_rel
            self.sources[repo_rel] = staged_rel

        add("docs/site/index.md", "index.md")
        pages_dir = os.path.join(self.site_dir, "pages")
        for name in sorted(os.listdir(pages_dir)):
            if name.endswith(".md"):
                add(f"docs/site/pages/{name}", name)

        eng = os.path.join(self.repo, "docs", "engineering")
        for root, dirs, files in os.walk(eng):
            dirs.sort()
            for name in sorted(files):
                if not name.endswith(".md"):
                    continue
                inside = posixpath.normpath(os.path.relpath(os.path.join(root, name), eng).replace(os.sep, "/"))
                staged_rel = "engineering/index.md" if inside == "README.md" else f"engineering/{inside}"
                add(self.repo_rel(os.path.join(root, name)), staged_rel)

        projects = os.path.join(self.repo, "projects")
        for project in sorted(os.listdir(projects)):
            pdocs = os.path.join(projects, project, "docs")
            if not os.path.isdir(pdocs):
                continue
            for name in sorted(os.listdir(pdocs)):
                if name.endswith(".md"):
                    add(f"projects/{project}/docs/{name}", f"{project}/{name.lower()}")

        # Generated index of the ADRs, built from the ADR files themselves.
        self.staged["engineering/adr/index.md"] = None

    def adr_index_text(self):
        adr_dir = os.path.join(self.repo, "docs", "engineering", "adr")
        rows = []
        for name in sorted(os.listdir(adr_dir)):
            if not name.endswith(".md") or name.startswith("0000-"):
                continue
            with open(os.path.join(adr_dir, name), encoding="utf-8") as handle:
                text = handle.read()
            title = first_heading(text) or name
            status = re.search(r"^- \*\*Status:\*\*\s*(.+?)\s*$", text, re.M)
            line = f"- [{title}]({name})"
            if status:
                line += f": {plain_text(status.group(1))}"
            rows.append(line)
        meta = {"title": "Architecture decision records", "layout": "default"}
        body = (
            "# Architecture decision records\n\n"
            "Decisions that change structure, tooling or a public contract. The source files are in "
            "`docs/engineering/adr/`. Start a new record from [the template](0000-template.md).\n\n"
            + "\n".join(rows)
            + "\n"
        )
        return dump_front_matter(meta) + body

    def resolve(self, target, repo_rel, staged_rel):
        """Returns the link target to write into the staged page, or raises LinkError."""
        if target.startswith("#") or EXTERNAL_RE.match(target):
            return target
        path, hash_sign, frag = target.partition("#")
        anchor = hash_sign + frag
        want_dir = path.endswith("/")
        base_dir = posixpath.dirname(staged_rel)

        # 1. Relative to the staged page (links between the engineering pages and the site).
        candidate = posixpath.normpath(posixpath.join(base_dir, path))
        hit = self.lookup_staged(candidate, want_dir)
        if hit:
            return posixpath.relpath(hit, base_dir or ".") + anchor

        # 2. Relative to the source file (links that point at repo files like ../../CONTRIBUTING.md).
        source_dir = posixpath.dirname(repo_rel) if repo_rel else ""
        repo_candidate = posixpath.normpath(posixpath.join(source_dir, path))
        if repo_candidate.startswith(".."):
            raise LinkError(f"link leaves the repository: {target}")
        if repo_candidate in self.sources and not want_dir:
            return posixpath.relpath(self.sources[repo_candidate], base_dir or ".") + anchor
        full = os.path.join(self.repo, repo_candidate)
        if os.path.isdir(full):
            return self.github("tree", repo_candidate) + anchor
        if os.path.isfile(full):
            return self.github("blob", repo_candidate) + anchor
        raise LinkError(f"no such page or file: {target}")

    def lookup_staged(self, candidate, want_dir):
        if not want_dir and candidate in self.staged:
            return candidate
        index = posixpath.join(candidate, "index.md")
        if index in self.staged:
            return index
        return None

    def github(self, kind, repo_rel):
        return f"{self.config['repo_url'].rstrip('/')}/{kind}/{GITHUB_BRANCH}/{repo_rel}"

    def rewrite_links(self, body, repo_rel, staged_rel):
        def rewrite_segment(segment):
            def repl(match):
                try:
                    new = self.resolve(match.group(2), repo_rel, staged_rel)
                except LinkError as exc:
                    self.error(staged_rel, str(exc))
                    new = match.group(2)
                return f"[{match.group(1)}]({new})"

            return LINK_RE.sub(repl, segment)

        out, fence = [], None
        for line in body.split("\n"):
            if fence is not None:
                closing = line.strip()
                if closing and set(closing) == {fence[0]} and len(closing) >= len(fence):
                    fence = None
                out.append(line)
                continue
            opening = FENCE_OPEN_RE.match(line)
            if opening:
                fence = opening.group(2)
                out.append(line)
                continue
            # Odd pieces of the split are code spans. Links inside code are left alone.
            pieces = CODE_SPAN_RE.split(line)
            out.append("".join(p if k % 2 else rewrite_segment(p) for k, p in enumerate(pieces)))
        return "\n".join(out)

    def substitute_site(self, body, where):
        return SITE_VAR_RE.sub(lambda m: self.site_value(m.group(1), where), body)

    def stage_sources(self):
        for staged_rel, repo_rel in sorted(self.staged.items()):
            if repo_rel is None:
                text = self.adr_index_text()
            else:
                with open(os.path.join(self.repo, repo_rel), encoding="utf-8") as handle:
                    text = handle.read()
            meta, body = split_front_matter(text)
            body = self.substitute_site(body, staged_rel)
            if repo_rel is not None:
                body = self.rewrite_links(body, repo_rel, staged_rel)
            title = meta.get("title") or first_heading(body) or posixpath.basename(staged_rel)
            front = {"title": plain_text(title), "layout": meta.get("layout", "default")}
            for key, value in meta.items():
                front.setdefault(key, value)
            self.pages[staged_rel] = (front, body)

            staged_text = dump_front_matter(front) + body.lstrip("\n")
            target = os.path.join(self.stage_dir, staged_rel)
            os.makedirs(os.path.dirname(target), exist_ok=True)
            with open(target, "w", encoding="utf-8") as handle:
                handle.write(staged_text)
            self.check_front_matter(staged_rel, staged_text)

    def check_front_matter(self, staged_rel, text):
        meta, _ = split_front_matter(text)
        for key in ("title", "layout"):
            if not meta.get(key):
                self.error(staged_rel, f"front matter has no '{key}'")

    # --- step 2: render

    def make_href(self, page):
        base = self.config["baseurl"]

        def convert(target):
            if target.startswith("#") or EXTERNAL_RE.match(target):
                return target
            path, hash_sign, frag = target.partition("#")
            if path.endswith(".md"):
                resolved = posixpath.normpath(posixpath.join(posixpath.dirname(page), path))
                return f"{base}/{resolved[:-3]}.html{hash_sign}{frag}"
            return target

        return convert

    def read_includes(self):
        includes = {}
        inc_dir = os.path.join(self.site_dir, "_includes")
        for name in os.listdir(inc_dir):
            with open(os.path.join(inc_dir, name), encoding="utf-8") as handle:
                includes[name] = handle.read()
        return includes

    def render(self):
        with open(os.path.join(self.site_dir, "_layouts", "default.html"), encoding="utf-8") as handle:
            layout = handle.read()
        includes = self.read_includes()

        def include_text(name):
            if name not in includes:
                self.error("layout", f"missing include '{name}'")
                return ""
            return includes[name]

        for staged_rel in sorted(self.pages):
            front, body = self.pages[staged_rel]
            md = Markdown(self.make_href(staged_rel))
            content = md.render(body)
            for code in md.mermaid:
                self.mermaid_blocks.append((staged_rel, code))

            title = front["title"]
            text = INCLUDE_RE.sub(lambda m: include_text(m.group(1)), layout)
            text = SITE_VAR_RE.sub(lambda m: html.escape(self.site_value(m.group(1), staged_rel), True), text)
            text = PAGE_VAR_RE.sub(lambda m: html.escape(title if m.group(1) == "title" else "", True), text)
            text = text.replace("{{ content }}", content)
            self.html_pages[staged_rel[:-3] + ".html"] = text

    # --- step 3: write and check

    def write(self):
        for rel, text in self.html_pages.items():
            target = os.path.join(self.out, rel)
            os.makedirs(os.path.dirname(target), exist_ok=True)
            with open(target, "w", encoding="utf-8") as handle:
                handle.write(text)
        shutil.copytree(os.path.join(self.site_dir, "assets"), os.path.join(self.out, "assets"), dirs_exist_ok=True)
        # Tells GitHub Pages not to run Jekyll over the generated files.
        with open(os.path.join(self.out, ".nojekyll"), "w", encoding="utf-8"):
            pass

    def check(self):
        for staged_rel, code in self.mermaid_blocks:
            problem = check_mermaid(code)
            if problem:
                self.error(staged_rel, problem)

        scans = {}
        for rel, text in self.html_pages.items():
            scan = PageScan()
            scan.feed(text)
            scans[rel] = scan

        base = self.config["baseurl"]
        prefix = base + "/"
        internal = 0
        for rel, scan in scans.items():
            for target in scan.targets:
                if target.startswith("#"):
                    if target[1:] not in scan.ids:
                        self.error(rel, f"same-page anchor not found: {target}")
                    continue
                if EXTERNAL_RE.match(target):
                    self.external_links += 1
                    continue
                if not target.startswith(prefix):
                    self.error(rel, f"link is not absolute within the site: {target}")
                    continue
                internal += 1
                path, _, frag = target[len(prefix) :].partition("#")
                if path == "" or path.endswith("/"):
                    path += "index.html"
                if path in scans:
                    if frag and frag not in scans[path].ids:
                        self.error(rel, f"anchor '{frag}' not found in {path}")
                elif os.path.isfile(os.path.join(self.out, path)):
                    continue
                else:
                    self.error(rel, f"broken internal link: {target}")
        self.internal_links = internal

    # --- driver

    def build(self):
        fresh_dir(self.stage_dir, self.repo, self.site_dir)
        fresh_dir(self.out, self.repo, self.site_dir)
        self.collect()
        self.stage_sources()
        self.render()
        self.write()
        self.check()
        if self.errors:
            for problem in self.errors:
                print(f"error: {problem}", file=sys.stderr)
            print(f"build failed with {len(self.errors)} error(s)", file=sys.stderr)
            return 1
        print(
            f"built {len(self.html_pages)} pages into {self.out}; "
            f"{self.internal_links} internal links checked, {self.external_links} external links left alone; "
            f"{len(self.mermaid_blocks)} Mermaid blocks passed the structure check"
        )
        return 0


def fresh_dir(path, repo, site_dir):
    """Empties a build directory. Refuses paths that would delete the repository or its site source."""
    path = os.path.abspath(path)
    if path in (repo, site_dir) or repo.startswith(path + os.sep) or path == os.path.dirname(site_dir):
        raise SystemExit(f"refusing to clear {path}")
    if os.path.isdir(path):
        shutil.rmtree(path)
    os.makedirs(path)


def main(argv=None):
    parser = argparse.ArgumentParser(description="Build the squirrel engineering docs site.")
    parser.add_argument("--repo-root", required=True)
    parser.add_argument("--site-dir", required=True)
    parser.add_argument("--stage", required=True)
    parser.add_argument("--out", required=True)
    args = parser.parse_args(argv)
    site = Site(args.repo_root, args.site_dir, args.stage, args.out)
    return site.build()


if __name__ == "__main__":
    sys.exit(main())
