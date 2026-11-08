// Tests for scripts/prepare-web.mjs. Run with `bun test test` from projects/axiom/desktop.
//
// The script finds ../web from its own location, so each test builds a small tree with the same
// shape (<root>/projects/axiom/desktop/scripts and <root>/projects/axiom/web) and runs the real
// script inside it. The real app-web/ folder is never touched.

import { test, expect } from "bun:test";
import { mkdirSync, mkdtempSync, writeFileSync, readFileSync, copyFileSync, existsSync, rmSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REAL_SCRIPT = path.resolve(HERE, "..", "scripts", "prepare-web.mjs");

// Builds the fixture tree, runs body(tree), and removes the tree afterwards.
function withTree({ wasm = false } = {}, body) {
  const root = mkdtempSync(path.join(os.tmpdir(), "axiom-prepare-web-"));
  try {
    const scriptDir = path.join(root, "projects", "axiom", "desktop", "scripts");
    const web = path.join(root, "projects", "axiom", "web");
    mkdirSync(scriptDir, { recursive: true });
    mkdirSync(path.join(web, "styles"), { recursive: true });
    const script = path.join(scriptDir, "prepare-web.mjs");
    copyFileSync(REAL_SCRIPT, script);

    writeFileSync(path.join(web, "index.html"), "<!doctype html><title>axiom</title>");
    writeFileSync(path.join(web, "input.js"), "export const x = 1;\n");
    writeFileSync(path.join(web, "input.test.js"), "// must not ship\n");
    writeFileSync(path.join(web, ".hidden"), "dotfile\n");
    writeFileSync(path.join(web, "styles", "main.css"), "body {}\n");
    if (wasm) writeFileSync(path.join(web, "axiom.wasm"), Buffer.from([0, 0x61, 0x73, 0x6d]));

    const dest = path.join(root, "projects", "axiom", "desktop", "app-web");
    return body({ script, scriptDir, web, dest });
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

// Runs the script with Bun from the scripts folder, the way `bun run prepare-web` does.
function run(tree, args = []) {
  const result = Bun.spawnSync([process.execPath, tree.script, ...args], {
    cwd: tree.scriptDir,
    stdout: "pipe",
    stderr: "pipe",
  });
  return {
    code: result.exitCode,
    out: result.stdout.toString(),
    err: result.stderr.toString(),
  };
}

test("fails when web/index.html is missing", () => {
  withTree({}, (tree) => {
    rmSync(path.join(tree.web, "index.html"));
    const result = run(tree, ["--allow-missing-wasm"]);
    expect(result.code).toBe(1);
    expect(result.err).toContain("index.html not found");
    expect(existsSync(tree.dest)).toBe(false);
  });
});

test("fails without axiom.wasm, and copies nothing", () => {
  withTree({ wasm: false }, (tree) => {
    const result = run(tree);
    expect(result.code).toBe(1);
    expect(result.err).toContain("axiom.wasm is missing");
    expect(existsSync(tree.dest)).toBe(false);
  });
});

test("--allow-missing-wasm copies the web files and warns", () => {
  withTree({ wasm: false }, (tree) => {
    const result = run(tree, ["--allow-missing-wasm"]);
    expect(result.code).toBe(0);
    expect(result.err).toContain("axiom.wasm is missing; packaging without the simulation");
    expect(existsSync(path.join(tree.dest, "index.html"))).toBe(true);
    expect(existsSync(path.join(tree.dest, "input.js"))).toBe(true);
    expect(existsSync(path.join(tree.dest, "styles", "main.css"))).toBe(true);
    expect(existsSync(path.join(tree.dest, "axiom.wasm"))).toBe(false);
  });
});

test("copies axiom.wasm byte for byte when it is present", () => {
  withTree({ wasm: true }, (tree) => {
    const result = run(tree);
    expect(result.code).toBe(0);
    expect(result.err).toBe("");
    const copied = readFileSync(path.join(tree.dest, "axiom.wasm"));
    expect(Array.from(copied)).toEqual([0, 0x61, 0x73, 0x6d]);
  });
});

test("never copies dotfiles or test files into the app", () => {
  withTree({ wasm: true }, (tree) => {
    expect(run(tree).code).toBe(0);
    expect(existsSync(path.join(tree.dest, ".hidden"))).toBe(false);
    expect(existsSync(path.join(tree.dest, "input.test.js"))).toBe(false);
    expect(existsSync(path.join(tree.dest, "input.js"))).toBe(true);
  });
});

test("a second run removes stale files from app-web", () => {
  withTree({ wasm: true }, (tree) => {
    mkdirSync(tree.dest, { recursive: true });
    writeFileSync(path.join(tree.dest, "stale.js"), "old build\n");
    expect(run(tree).code).toBe(0);
    expect(existsSync(path.join(tree.dest, "stale.js"))).toBe(false);
    expect(existsSync(path.join(tree.dest, "index.html"))).toBe(true);
  });
});
