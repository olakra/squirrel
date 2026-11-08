#!/usr/bin/env python3
"""Fills the steamcmd templates in this folder from environment variables.

Each ${NAME} token in app_build.vdf and depot_build_*.vdf is replaced by the value of
the environment variable NAME. The script refuses to write anything when a token has no
value, so a half-filled upload never starts. It also checks that every *_ID value is
numeric. Values are never printed; error messages name variables only.

Usage:
    python3 steam/render_vdf.py --out DIR

Required environment:
    STEAM_APP_ID, STEAM_DEPOT_WINDOWS_ID, STEAM_DEPOT_LINUX_ID, STEAM_DEPOT_MACOS_ID,
    AXIOM_STAGING_DIR (absolute path that contains windows/, linux/ and macos/ folders)

Optional environment:
    AXIOM_BUILD_DESC (default "unversioned"), AXIOM_BUILD_OUTPUT (default DIR/output),
    AXIOM_SET_LIVE (default "", which uploads to the preview build without setting it live)
"""

import argparse
import os
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
TEMPLATES = ["app_build.vdf", "depot_build_windows.vdf", "depot_build_linux.vdf", "depot_build_macos.vdf"]
REQUIRED = [
    "STEAM_APP_ID",
    "STEAM_DEPOT_WINDOWS_ID",
    "STEAM_DEPOT_LINUX_ID",
    "STEAM_DEPOT_MACOS_ID",
    "AXIOM_STAGING_DIR",
]
TOKEN = re.compile(r"\$\{([A-Z0-9_]+)\}")


def fail(message: str) -> None:
    print(f"error: {message}", file=sys.stderr)
    sys.exit(1)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--out", required=True, type=Path, help="folder to write the rendered files into")
    args = parser.parse_args()
    out_dir = args.out.resolve()

    values = {}
    missing = [name for name in REQUIRED if not os.environ.get(name, "").strip()]
    if missing:
        fail("missing required environment variables: " + ", ".join(missing))
    for name in REQUIRED:
        values[name] = os.environ[name].strip()

    for name in ("STEAM_APP_ID", "STEAM_DEPOT_WINDOWS_ID", "STEAM_DEPOT_LINUX_ID", "STEAM_DEPOT_MACOS_ID"):
        if not values[name].isdigit():
            fail(f"{name} must contain digits only")

    staging = Path(values["AXIOM_STAGING_DIR"])
    if not staging.is_absolute():
        fail("AXIOM_STAGING_DIR must be an absolute path")

    values["AXIOM_BUILD_DESC"] = os.environ.get("AXIOM_BUILD_DESC", "").strip() or "unversioned"
    values["AXIOM_SET_LIVE"] = os.environ.get("AXIOM_SET_LIVE", "").strip()
    values["AXIOM_BUILD_OUTPUT"] = os.environ.get("AXIOM_BUILD_OUTPUT", "").strip() or str(out_dir / "output")
    values["AXIOM_STAGING_DIR"] = str(staging)

    # Values must not contain quotes or newlines: they would break the VDF syntax.
    for name, value in values.items():
        if any(ch in value for ch in '"\n\r'):
            fail(f"{name} contains a quote or newline, which VDF cannot hold")

    # Check every template before writing anything, so a bad value leaves no partial output.
    rendered = {}
    unresolved = set()
    for template_name in TEMPLATES:
        text = (HERE / template_name).read_text(encoding="utf-8")
        unresolved.update(t for t in TOKEN.findall(text) if t not in values)
        rendered[template_name] = TOKEN.sub(lambda m: values.get(m.group(1), m.group(0)), text)
    if unresolved:
        fail("templates use tokens with no value: " + ", ".join(sorted(unresolved)))

    out_dir.mkdir(parents=True, exist_ok=True)
    for template_name, text in rendered.items():
        target = out_dir / template_name
        target.write_text(text, encoding="utf-8")
        os.chmod(target, 0o600)

    print(f"rendered {len(TEMPLATES)} files into {out_dir}")


if __name__ == "__main__":
    main()
