// Stages one platform's unpacked build into desktop/dist/staging/<platform>/, the layout
// that steam/render_vdf.py expects as a depot ContentRoot.
//
// Usage: node scripts/stage-depot.mjs <windows|linux|macos>
// Exits non-zero when the expected output folder or executable is missing.

import { access, cp, mkdir, rm } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const distDir = path.resolve(here, "..", "dist");

// electron-builder names the unpacked folder after the target and arch. Try each known name.
const PLATFORMS = {
  windows: { candidates: ["win-unpacked"], entry: "Axiom.exe" },
  linux: { candidates: ["linux-unpacked"], entry: "axiom" },
  macos: { candidates: ["mac-universal", "mac-arm64", "mac-x64", "mac"], entry: "Axiom.app" },
};

async function exists(p) {
  try {
    await access(p);
    return true;
  } catch {
    return false;
  }
}

const platform = process.argv[2];
const spec = PLATFORMS[platform];
if (!spec) {
  console.error("usage: node scripts/stage-depot.mjs <windows|linux|macos>");
  process.exit(2);
}

let source = null;
for (const name of spec.candidates) {
  const candidate = path.join(distDir, name);
  if (await exists(candidate)) {
    source = candidate;
    break;
  }
}
if (!source) {
  console.error(`error: no build output for ${platform} under ${distDir} (tried ${spec.candidates.join(", ")})`);
  process.exit(1);
}
if (!(await exists(path.join(source, spec.entry)))) {
  console.error(`error: ${path.join(source, spec.entry)} is missing; the build is incomplete`);
  process.exit(1);
}

const dest = path.join(distDir, "staging", platform);
await rm(dest, { recursive: true, force: true });
await mkdir(path.dirname(dest), { recursive: true });
// cp keeps symlinks as symlinks by default, which the macOS framework layout needs.
await cp(source, dest, { recursive: true });
console.log(`staged ${source} to ${dest}`);
