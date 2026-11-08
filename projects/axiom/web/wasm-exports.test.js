// Headless tests for the WASM export layer (src/wasm.rs). Run with `bun test`
// from projects/axiom/web, or through `make js-test`.
//
// The tests load web/axiom.wasm directly, with no browser. When that file is
// absent (the wasm32 build has not been run) every test is skipped, not failed.

import { test, expect } from "bun:test";
import { existsSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const WASM_PATH = fileURLToPath(new URL("./axiom.wasm", import.meta.url));
const HAS_WASM = existsSync(WASM_PATH);

// Same layout the browser worker uses.
const LANE_COUNT = 2;
const LANE_LENGTH = 40;
const LANE_SPEED = 2;

let compiled = null;

// Compiles once, then instantiates a fresh module per call. Each instance has
// its own linear memory, so each one has its own world.
async function freshExports() {
  if (!compiled) compiled = WebAssembly.compile(readFileSync(WASM_PATH));
  const instance = await WebAssembly.instantiate(await compiled, {});
  return instance.exports;
}

function stateHash(ex) {
  return (BigInt(ex.axiom_state_hash_hi()) << 32n) | BigInt(ex.axiom_state_hash_lo());
}

test.skipIf(!HAS_WASM)("init with 2 lanes, then 100 steps, gives tick_count 100", async () => {
  const ex = await freshExports();
  ex.axiom_init(LANE_COUNT, LANE_LENGTH, LANE_SPEED);
  expect(ex.axiom_tick_count()).toBe(0);
  ex.axiom_step(100);
  expect(ex.axiom_tick_count()).toBe(100);
});

test.skipIf(!HAS_WASM)("axiom_feed accepts an item on an empty head lane", async () => {
  const ex = await freshExports();
  ex.axiom_init(LANE_COUNT, LANE_LENGTH, LANE_SPEED);
  expect(ex.axiom_feed(1)).toBe(1);
});

test.skipIf(!HAS_WASM)("axiom_snapshot_items reports a fed item with its id, lane and offset", async () => {
  const ex = await freshExports();
  ex.axiom_init(LANE_COUNT, LANE_LENGTH, LANE_SPEED);
  expect(ex.axiom_snapshot_items()).toBe(0);
  expect(ex.axiom_feed(5)).toBe(1);
  ex.axiom_step(3);
  expect(ex.axiom_snapshot_items()).toBe(1);
  expect(ex.axiom_item_id(0)).toBe(5);
  expect(ex.axiom_item_lane(0)).toBe(0);
  expect(ex.axiom_item_offset(0)).toBe(LANE_SPEED * 3);
  // An index past the end reads as 0 instead of failing.
  expect(ex.axiom_item_id(99)).toBe(0);
});

test.skipIf(!HAS_WASM)("axiom_feed rejects an item id above u16::MAX", async () => {
  const ex = await freshExports();
  ex.axiom_init(LANE_COUNT, LANE_LENGTH, LANE_SPEED);
  expect(ex.axiom_feed(0x1_0000)).toBe(0);
});

test.skipIf(!HAS_WASM)("state hash is identical across two fresh instances fed the same sequence", async () => {
  const run = async () => {
    const ex = await freshExports();
    ex.axiom_init(LANE_COUNT, LANE_LENGTH, LANE_SPEED);
    for (let i = 1; i <= 10; i++) {
      ex.axiom_feed(i);
      ex.axiom_step(5);
    }
    return ex;
  };
  const a = await run();
  const b = await run();
  expect(stateHash(a)).toEqual(stateHash(b));
  expect(a.axiom_tick_count()).toBe(50);
});

test.skipIf(!HAS_WASM)("axiom_sunk returns a number", async () => {
  const ex = await freshExports();
  ex.axiom_init(LANE_COUNT, LANE_LENGTH, LANE_SPEED);
  ex.axiom_step(100);
  expect(typeof ex.axiom_sunk()).toBe("number");
});
