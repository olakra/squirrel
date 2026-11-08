// Copies ../web into desktop/app-web so electron-builder can bundle it.
//
// Fails when axiom.wasm is missing, because a release without the simulation is a broken
// release. For a local smoke test without WASM, pass --allow-missing-wasm.

import { access, cp, rm } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const src = path.resolve(here, "..", "..", "web");
const dest = path.resolve(here, "..", "app-web");
const allowMissingWasm = process.argv.includes("--allow-missing-wasm");

async function exists(p) {
  try {
    await access(p);
    return true;
  } catch {
    return false;
  }
}

if (!(await exists(path.join(src, "index.html")))) {
  console.error(`error: ${path.join(src, "index.html")} not found`);
  process.exit(1);
}

if (!(await exists(path.join(src, "axiom.wasm")))) {
  if (!allowMissingWasm) {
    console.error("error: web/axiom.wasm is missing. Run projects/axiom/scripts/build-wasm.sh first.");
    console.error("       (pass --allow-missing-wasm only for a local smoke test)");
    process.exit(1);
  }
  console.warn("warning: web/axiom.wasm is missing; packaging without the simulation");
}

await rm(dest, { recursive: true, force: true });
await cp(src, dest, {
  recursive: true,
  filter: (p) => !path.basename(p).startsWith("."),
});
console.log(`copied ${src} to ${dest}`);
