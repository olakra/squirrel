// Tests for the pure helpers in renderer.js. Run with `bun test` from projects/axiom/web.
// The GL path needs a browser and is not covered here.

import { test, expect, describe } from "bun:test";
import {
  FLOATS_PER_INSTANCE,
  BYTES_PER_INSTANCE,
  SHAPE,
  packInstance,
  clampCount,
  colourForItem,
  shapeForItem,
  parseHex,
  parseBackground,
} from "./renderer.js";

describe("packInstance", () => {
  test("writes x, y, w, h, r, g, b, a in that order at the given index", () => {
    const buf = new Float32Array(FLOATS_PER_INSTANCE * 3);
    packInstance(buf, 1, { x: 0.25, y: 0.5, w: 0.125, h: 0.0625 }, { r: 0.1, g: 0.2, b: 0.3, a: 0.4 });
    const o = FLOATS_PER_INSTANCE;
    expect(Array.from(buf.slice(o, o + 8))).toEqual(
      [0.25, 0.5, 0.125, 0.0625, 0.1, 0.2, 0.3, 0.4].map((v) => Math.fround(v)),
    );
  });

  test("does not touch neighbouring instances", () => {
    const buf = new Float32Array(FLOATS_PER_INSTANCE * 3).fill(-1);
    packInstance(buf, 1, { x: 1, y: 1, w: 1, h: 1 }, { r: 1, g: 1, b: 1, a: 1 });
    expect(buf.slice(0, FLOATS_PER_INSTANCE).every((v) => v === -1)).toBe(true);
    expect(buf.slice(2 * FLOATS_PER_INSTANCE).every((v) => v === -1)).toBe(true);
  });

  test("returns the same buffer", () => {
    const buf = new Float32Array(FLOATS_PER_INSTANCE);
    expect(packInstance(buf, 0, { x: 0, y: 0, w: 0, h: 0 }, { r: 0, g: 0, b: 0, a: 0 })).toBe(buf);
  });

  test("instance stride matches the 32-byte GL attribute layout", () => {
    expect(FLOATS_PER_INSTANCE).toBe(8);
    expect(BYTES_PER_INSTANCE).toBe(32);
  });
});

describe("clampCount", () => {
  test("passes through counts within capacity", () => {
    expect(clampCount(0, 10)).toBe(0);
    expect(clampCount(7, 10)).toBe(7);
    expect(clampCount(10, 10)).toBe(10);
  });

  test("caps counts above capacity", () => {
    expect(clampCount(11, 10)).toBe(10);
    expect(clampCount(1e9, 4096)).toBe(4096);
  });

  test("maps negative, NaN and infinite counts to a safe value", () => {
    expect(clampCount(-5, 10)).toBe(0);
    expect(clampCount(NaN, 10)).toBe(0);
    expect(clampCount(Infinity, 10)).toBe(0);
    expect(clampCount(-Infinity, 10)).toBe(0);
  });

  test("floors fractional counts", () => {
    expect(clampCount(3.9, 10)).toBe(3);
  });

  test("gives 0 for a zero or invalid capacity", () => {
    expect(clampCount(5, 0)).toBe(0);
    expect(clampCount(5, -1)).toBe(0);
    expect(clampCount(5, NaN)).toBe(0);
  });
});

describe("palette", () => {
  test("colourForItem is deterministic", () => {
    for (let id = -20; id <= 20; id++) {
      expect(colourForItem(id)).toEqual(colourForItem(id));
    }
  });

  test("colourForItem returns RGBA floats in 0..1", () => {
    for (let id = -20; id <= 20; id++) {
      const c = colourForItem(id);
      for (const k of ["r", "g", "b", "a"]) {
        expect(c[k]).toBeGreaterThanOrEqual(0);
        expect(c[k]).toBeLessThanOrEqual(1);
      }
    }
  });

  test("colourForItem wraps ids onto the same fixed table", () => {
    const seen = new Set();
    for (let id = 0; id < 6; id++) {
      const c = colourForItem(id);
      seen.add(`${c.r},${c.g},${c.b}`);
      expect(colourForItem(id + 6)).toEqual(c);
    }
    expect(seen.size).toBe(6);
  });

  test("non-finite ids fall back to class 0 instead of throwing", () => {
    expect(colourForItem(NaN)).toEqual(colourForItem(0));
    expect(shapeForItem(Infinity)).toBe(shapeForItem(0));
  });

  test("shape ids are integers in 0..2", () => {
    for (let id = -20; id <= 20; id++) {
      const s = shapeForItem(id);
      expect(Number.isInteger(s)).toBe(true);
      expect(s).toBeGreaterThanOrEqual(0);
      expect(s).toBeLessThanOrEqual(2);
    }
  });

  test("SHAPE constants are 0, 1 and 2", () => {
    expect(SHAPE).toEqual({ SQUARE: 0, DIAMOND: 1, CIRCLE: 2 });
  });

  test("item classes differ in shape as well as colour, so colour is not the only cue", () => {
    const shapes = new Set();
    for (let id = 0; id < 6; id++) shapes.add(shapeForItem(id));
    expect(shapes.size).toBeGreaterThan(1);
  });
});

describe("colour parsing", () => {
  test("parseHex reads #RRGGBB", () => {
    expect(parseHex("#FFFFFF")).toEqual({ r: 1, g: 1, b: 1, a: 1 });
    expect(parseHex("#000000")).toEqual({ r: 0, g: 0, b: 0, a: 1 });
  });

  test("parseHex rejects malformed input", () => {
    expect(parseHex("FFFFFF")).toBeNull();
    expect(parseHex("#FFF")).toBeNull();
    expect(parseHex(123)).toBeNull();
  });

  test("parseBackground accepts hex, arrays, and falls back to the factory floor", () => {
    expect(parseBackground("#1E222A")).toEqual(parseHex("#1E222A"));
    expect(parseBackground([0.5, 0.25, 2, 0.5])).toEqual({ r: 0.5, g: 0.25, b: 1, a: 0.5 });
    expect(parseBackground([0, 0, 0])).toEqual({ r: 0, g: 0, b: 0, a: 1 });
    expect(parseBackground("nonsense")).toEqual(parseHex("#1E222A"));
    expect(parseBackground(undefined)).toEqual(parseHex("#1E222A"));
  });
});
