// Tests for input.js. Run with `bun test` from projects/axiom/web.
// The module is pure logic, so these tests need no browser and no network.

import { test, expect } from "bun:test";
import {
  PAD,
  DEADZONE,
  STEP_PX,
  STICK_SPEED_PX,
  REPEAT_DELAY_MS,
  REPEAT_INTERVAL_MS,
  applyDeadzone,
  createGamepadInput,
  edgeIntent,
  keyToIntent,
} from "./input.js";

// Builds a standard-mapping pad with the given buttons held and the given axes.
function makePad(pressed = [], axes = [0, 0]) {
  const buttons = Array.from({ length: 17 }, (_, i) => ({
    pressed: pressed.includes(i),
    value: pressed.includes(i) ? 1 : 0,
  }));
  return { connected: true, buttons, axes };
}

// Returns a poller whose pads the test can change between frames.
function harness() {
  const state = { pads: [] };
  const input = createGamepadInput(() => state.pads);
  return {
    input,
    set(pressed, axes) {
      state.pads = [makePad(pressed, axes)];
    },
    setPads(pads) {
      state.pads = pads;
    },
  };
}

// --- applyDeadzone -----------------------------------------------------------

test("applyDeadzone returns 0 inside and exactly at the deadzone", () => {
  expect(applyDeadzone(0)).toBe(0);
  expect(applyDeadzone(DEADZONE)).toBe(0);
  expect(applyDeadzone(-DEADZONE)).toBe(0);
  expect(applyDeadzone(0.15)).toBe(0);
  expect(applyDeadzone(-0.15)).toBe(0);
});

test("applyDeadzone starts near zero just outside the deadzone", () => {
  expect(applyDeadzone(DEADZONE + 0.0001)).toBeCloseTo(0, 3);
});

test("applyDeadzone reaches 1 at full deflection and keeps the sign", () => {
  expect(applyDeadzone(1)).toBe(1);
  expect(applyDeadzone(-1)).toBe(-1);
  expect(applyDeadzone(0.6)).toBeCloseTo(0.5, 10);
  expect(applyDeadzone(-0.6)).toBeCloseTo(-0.5, 10);
});

test("applyDeadzone clamps out-of-range and rejects non-finite values", () => {
  expect(applyDeadzone(1.5)).toBe(1);
  expect(applyDeadzone(Number.NaN)).toBe(0);
  expect(applyDeadzone(Number.POSITIVE_INFINITY)).toBe(0);
});

test("applyDeadzone accepts a custom deadzone", () => {
  expect(applyDeadzone(0.4, 0.5)).toBe(0);
  expect(applyDeadzone(0.75, 0.5)).toBeCloseTo(0.5, 10);
});

// --- edgeIntent and keyToIntent ----------------------------------------------

test("edgeIntent maps A, B and Start and nothing else", () => {
  expect(edgeIntent(PAD.A)).toEqual({ type: "confirm" });
  expect(edgeIntent(PAD.B)).toEqual({ type: "cancel" });
  expect(edgeIntent(PAD.START)).toEqual({ type: "toggleHints" });
  expect(edgeIntent(PAD.X)).toBeNull();
  expect(edgeIntent(PAD.UP)).toBeNull();
});

test("keyToIntent maps arrows and WASD to one cursor step", () => {
  expect(keyToIntent("ArrowUp")).toEqual({ type: "move", dx: 0, dy: -STEP_PX });
  expect(keyToIntent("KeyW")).toEqual({ type: "move", dx: 0, dy: -STEP_PX });
  expect(keyToIntent("ArrowDown")).toEqual({ type: "move", dx: 0, dy: STEP_PX });
  expect(keyToIntent("KeyA")).toEqual({ type: "move", dx: -STEP_PX, dy: 0 });
  expect(keyToIntent("ArrowRight")).toEqual({ type: "move", dx: STEP_PX, dy: 0 });
});

test("keyToIntent maps confirm, cancel and hints keys", () => {
  expect(keyToIntent("Enter")).toEqual({ type: "confirm" });
  expect(keyToIntent("Space")).toEqual({ type: "confirm" });
  expect(keyToIntent("Escape")).toEqual({ type: "cancel" });
  expect(keyToIntent("KeyH")).toEqual({ type: "toggleHints" });
  expect(keyToIntent("KeyQ")).toBeNull();
});

// --- edge-triggered buttons --------------------------------------------------

test("a button emits once per press, not once per frame", () => {
  const h = harness();
  h.set([PAD.A]);
  expect(h.input.poll(0).intents).toEqual([{ type: "confirm" }]);

  // Still held on later frames: no new intent.
  expect(h.input.poll(16).intents).toEqual([]);
  expect(h.input.poll(32).intents).toEqual([]);

  // Release, then press again: one more intent.
  h.set([]);
  expect(h.input.poll(48).intents).toEqual([]);
  h.set([PAD.A]);
  expect(h.input.poll(64).intents).toEqual([{ type: "confirm" }]);
});

test("a press of an unmapped button marks activity but emits no intent", () => {
  const h = harness();
  h.set([PAD.X]);
  const result = h.input.poll(0);
  expect(result.intents).toEqual([]);
  expect(result.active).toBe(true);
});

test("two buttons pressed in the same frame both emit", () => {
  const h = harness();
  h.set([PAD.B, PAD.START]);
  const intents = h.input.poll(0).intents;
  expect(intents).toContainEqual({ type: "cancel" });
  expect(intents).toContainEqual({ type: "toggleHints" });
  expect(intents.length).toBe(2);
});

test("disconnected and missing pads are ignored", () => {
  const h = harness();
  h.setPads([{ connected: false, buttons: [{ pressed: true }], axes: [1, 0] }, null]);
  expect(h.input.poll(0)).toEqual({ intents: [], active: false });
  expect(h.input.poll(50)).toEqual({ intents: [], active: false });
});

// --- left stick --------------------------------------------------------------

test("left stick moves the cursor in proportion to deflection and frame time", () => {
  const h = harness();
  h.set([], [1, 0]);
  h.input.poll(0); // first frame has no elapsed time
  const intents = h.input.poll(50).intents;
  // 50 ms at full speed: STICK_SPEED_PX per second times 0.05 s.
  expect(intents).toEqual([{ type: "move", dx: (STICK_SPEED_PX * 50) / 1000, dy: 0 }]);
});

test("stick drift inside the deadzone produces no movement", () => {
  const h = harness();
  h.set([], [0.2, -0.1]);
  h.input.poll(0);
  expect(h.input.poll(100).intents).toEqual([]);
});

test("a backgrounded tab cannot jump the cursor: frame time is capped", () => {
  const h = harness();
  h.set([], [1, 0]);
  h.input.poll(0);
  // A minute later, the frame delta is capped at 100 ms, so the move is 100 ms of travel.
  const intents = h.input.poll(60_000).intents;
  expect(intents).toEqual([{ type: "move", dx: (STICK_SPEED_PX * 100) / 1000, dy: 0 }]);
});

// --- D-pad -------------------------------------------------------------------

test("D-pad moves one step on press, waits, then repeats while held", () => {
  const h = harness();
  h.set([PAD.UP]);
  expect(h.input.poll(1000).intents).toEqual([{ type: "move", dx: 0, dy: -STEP_PX }]);

  // Before the repeat delay: nothing.
  expect(h.input.poll(1000 + REPEAT_DELAY_MS - 150).intents).toEqual([]);

  // At the repeat delay: one step, then steps at the repeat interval.
  expect(h.input.poll(1000 + REPEAT_DELAY_MS).intents).toEqual([{ type: "move", dx: 0, dy: -STEP_PX }]);
  const next = 1000 + REPEAT_DELAY_MS + REPEAT_INTERVAL_MS;
  expect(h.input.poll(next - 10).intents).toEqual([]);
  expect(h.input.poll(next).intents).toEqual([{ type: "move", dx: 0, dy: -STEP_PX }]);
});

test("D-pad release resets the repeat, so the next press steps at once", () => {
  const h = harness();
  h.set([PAD.LEFT]);
  h.input.poll(0);
  h.set([]);
  h.input.poll(20);
  h.set([PAD.LEFT]);
  expect(h.input.poll(40).intents).toEqual([{ type: "move", dx: -STEP_PX, dy: 0 }]);
});

test("D-pad directions map to the right deltas", () => {
  const cases = [
    [PAD.DOWN, 0, STEP_PX],
    [PAD.LEFT, -STEP_PX, 0],
    [PAD.RIGHT, STEP_PX, 0],
  ];
  for (const [button, dx, dy] of cases) {
    const h = harness();
    h.set([button]);
    expect(h.input.poll(0).intents).toEqual([{ type: "move", dx, dy }]);
  }
});
