// Tests for items.js. Run with `bun test` from projects/axiom/web.
// Pure logic: the renderer helpers are real, the GL path is not used.

import { test, expect, describe } from "bun:test";
import { FLOATS_PER_INSTANCE as RENDERER_FLOATS, packInstance, colourForItem } from "./renderer.js";
import { FLOATS_PER_INSTANCE, LAYOUT, itemRect, writeInstances } from "./items.js";

const LANE_LENGTH = 40;

describe("layout constants", () => {
  test("instance width matches the renderer", () => {
    expect(FLOATS_PER_INSTANCE).toBe(RENDERER_FLOATS);
  });
});

describe("itemRect", () => {
  test("an item at the rear sits at the left end of its lane", () => {
    const r = itemRect(0, 0, LANE_LENGTH);
    expect(r.x).toBeCloseTo(LAYOUT.xStart - LAYOUT.itemW / 2, 10);
  });

  test("an item at the lane end sits at the right end", () => {
    const r = itemRect(0, LANE_LENGTH, LANE_LENGTH);
    expect(r.x).toBeCloseTo(LAYOUT.xEnd - LAYOUT.itemW / 2, 10);
  });

  test("items move right as their offset grows", () => {
    const a = itemRect(0, 4, LANE_LENGTH);
    const b = itemRect(0, 12, LANE_LENGTH);
    expect(b.x).toBeGreaterThan(a.x);
  });

  test("lane 1 sits below lane 0", () => {
    const top = itemRect(0, 10, LANE_LENGTH);
    const next = itemRect(1, 10, LANE_LENGTH);
    expect(next.y).toBeCloseTo(top.y + LAYOUT.rowGap, 10);
  });

  test("offsets beyond the lane are clamped to its end", () => {
    expect(itemRect(0, 999, LANE_LENGTH).x).toBeCloseTo(itemRect(0, LANE_LENGTH, LANE_LENGTH).x, 10);
  });

  test("a zero-length lane does not divide by zero", () => {
    expect(Number.isFinite(itemRect(0, 5, 0).x)).toBe(true);
  });

  test("rectangles stay in the 0..1 canvas range for every valid offset", () => {
    for (let lane = 0; lane < 2; lane++) {
      for (let offset = 0; offset <= LANE_LENGTH; offset++) {
        const r = itemRect(lane, offset, LANE_LENGTH);
        expect(r.x).toBeGreaterThanOrEqual(0);
        expect(r.x + r.w).toBeLessThanOrEqual(1);
        expect(r.y).toBeGreaterThanOrEqual(0);
        expect(r.y + r.h).toBeLessThanOrEqual(1);
      }
    }
  });
});

describe("writeInstances", () => {
  const items = [
    { id: 1, lane: 0, offset: 0 },
    { id: 2, lane: 1, offset: 20 },
  ];
  const read = {
    id: (i) => items[i].id,
    lane: (i) => items[i].lane,
    offset: (i) => items[i].offset,
  };

  test("writes one instance per item with the item's colour", () => {
    const buffer = new Float32Array(items.length * FLOATS_PER_INSTANCE);
    const n = writeInstances(buffer, items.length, read, LANE_LENGTH, packInstance, colourForItem);
    expect(n).toBe(2);
    const c = colourForItem(2);
    expect(buffer[FLOATS_PER_INSTANCE + 4]).toBeCloseTo(c.r, 5);
    expect(buffer[FLOATS_PER_INSTANCE + 5]).toBeCloseTo(c.g, 5);
  });

  test("stops at the buffer capacity instead of writing past it", () => {
    const buffer = new Float32Array(FLOATS_PER_INSTANCE);
    const n = writeInstances(buffer, items.length, read, LANE_LENGTH, packInstance, colourForItem);
    expect(n).toBe(1);
  });

  test("writes nothing for a zero count", () => {
    const buffer = new Float32Array(FLOATS_PER_INSTANCE);
    expect(writeInstances(buffer, 0, read, LANE_LENGTH, packInstance, colourForItem)).toBe(0);
  });

  test("a negative or fractional count is treated as a whole, non-negative count", () => {
    const buffer = new Float32Array(FLOATS_PER_INSTANCE * 4);
    expect(writeInstances(buffer, -3, read, LANE_LENGTH, packInstance, colourForItem)).toBe(0);
    expect(writeInstances(buffer, 1.9, read, LANE_LENGTH, packInstance, colourForItem)).toBe(1);
  });
});
