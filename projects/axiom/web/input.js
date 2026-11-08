// Input translation for keyboard and gamepad.
//
// This module is pure logic. It reads gamepad state and key codes, and it
// returns intents. It never touches the simulation, the worker, or the DOM.
// The caller (main.js) decides what each intent means. Simulation state is
// changed only by the worker, after the caller forwards a game intent.
//
// Intent shapes:
//   { type: "move", dx, dy }   cursor move in CSS pixels (UI only)
//   { type: "confirm" }        A button or Enter
//   { type: "cancel" }         B button or Escape
//   { type: "toggleHints" }    Start button or H
//
// Button indices follow the W3C "standard" Gamepad mapping, which is the Xbox
// layout. Steam Input presents controllers this way, including the Steam Deck.

export const PAD = Object.freeze({
  A: 0,
  B: 1,
  X: 2,
  Y: 3,
  LB: 4,
  RB: 5,
  BACK: 8,
  START: 9,
  LS: 10,
  RS: 11,
  UP: 12,
  DOWN: 13,
  LEFT: 14,
  RIGHT: 15,
  HOME: 16,
});

// Stick deflection below this fraction is ignored (drift protection).
export const DEADZONE = 0.2;
// Cursor movement per D-pad or key step, in CSS pixels. Matches a 16 px grid.
export const STEP_PX = 16;
// Stick speed at full deflection, in CSS pixels per second.
export const STICK_SPEED_PX = 480;
// Auto-repeat for held D-pad directions: first repeat after the delay, then at the interval.
export const REPEAT_DELAY_MS = 350;
export const REPEAT_INTERVAL_MS = 90;

// Largest frame delta we honour. A tab that was backgrounded must not jump the cursor.
const MAX_DT_MS = 100;

const DPAD_DIRS = [
  [PAD.UP, 0, -1],
  [PAD.DOWN, 0, 1],
  [PAD.LEFT, -1, 0],
  [PAD.RIGHT, 1, 0],
];

// Rescales a stick axis so the output starts at 0 just outside the deadzone and reaches 1 at full deflection.
export function applyDeadzone(value, deadzone = DEADZONE) {
  const magnitude = Math.abs(value);
  if (!Number.isFinite(value) || magnitude <= deadzone) return 0;
  const scaled = Math.min(1, (magnitude - deadzone) / (1 - deadzone));
  return Math.sign(value) * scaled;
}

// Returns a poller. Call poll(now) once per animation frame, with a monotonic timestamp in ms.
// `getPads` is injectable so the logic can be tested without a browser.
export function createGamepadInput(getPads = defaultGetPads) {
  const pressedLast = new Map(); // pad index -> Set of button indices pressed last frame
  const repeats = new Map(); // "pad:button" -> { next, count }
  let lastNow = null;

  function poll(now) {
    const dt = lastNow === null ? 0 : Math.min(MAX_DT_MS, Math.max(0, now - lastNow));
    lastNow = now;
    const intents = [];
    let active = false;

    const pads = Array.from(getPads() || []);
    pads.forEach((pad, index) => {
      if (!pad || !pad.connected) return;
      const buttons = pad.buttons || [];
      const axes = pad.axes || [];
      const isDown = (b) => Boolean(buttons[b] && buttons[b].pressed);

      // Edge detection: a button emits once per press, not once per frame.
      const previous = pressedLast.get(index) || new Set();
      const current = new Set();
      buttons.forEach((button, b) => {
        if (button && button.pressed) current.add(b);
      });
      for (const b of current) {
        if (previous.has(b)) continue;
        active = true;
        const intent = edgeIntent(b);
        if (intent) intents.push(intent);
      }
      pressedLast.set(index, current);

      // Left stick: continuous movement scaled by deflection.
      const sx = applyDeadzone(axes[0] || 0);
      const sy = applyDeadzone(axes[1] || 0);
      if (sx !== 0 || sy !== 0) {
        active = true;
        const scale = (STICK_SPEED_PX * dt) / 1000;
        intents.push({ type: "move", dx: sx * scale, dy: sy * scale });
      }

      // D-pad: one step on press, then auto-repeat while held.
      for (const [b, dx, dy] of DPAD_DIRS) {
        const key = `${index}:${b}`;
        if (!isDown(b)) {
          repeats.delete(key);
          continue;
        }
        active = true;
        let state = repeats.get(key);
        if (!state) {
          state = { next: now, count: 0 };
          repeats.set(key, state);
        }
        if (now >= state.next) {
          intents.push({ type: "move", dx: dx * STEP_PX, dy: dy * STEP_PX });
          state.next = now + (state.count === 0 ? REPEAT_DELAY_MS : REPEAT_INTERVAL_MS);
          state.count += 1;
        }
      }
    });

    return { intents, active };
  }

  return { poll };
}

// Maps a button that was just pressed to its intent. Returns null for unmapped buttons.
export function edgeIntent(button) {
  switch (button) {
    case PAD.A:
      return { type: "confirm" };
    case PAD.B:
      return { type: "cancel" };
    case PAD.START:
      return { type: "toggleHints" };
    default:
      return null;
  }
}

// Maps a KeyboardEvent.code to an intent. Returns null for unmapped keys.
export function keyToIntent(code) {
  switch (code) {
    case "ArrowUp":
    case "KeyW":
      return { type: "move", dx: 0, dy: -STEP_PX };
    case "ArrowDown":
    case "KeyS":
      return { type: "move", dx: 0, dy: STEP_PX };
    case "ArrowLeft":
    case "KeyA":
      return { type: "move", dx: -STEP_PX, dy: 0 };
    case "ArrowRight":
    case "KeyD":
      return { type: "move", dx: STEP_PX, dy: 0 };
    case "Enter":
    case "Space":
      return { type: "confirm" };
    case "Escape":
      return { type: "cancel" };
    case "KeyH":
      return { type: "toggleHints" };
    default:
      return null;
  }
}

// On-screen hint rows, one set per input device. Glyph text is what the
// player sees, so keep it in step with the mapping above.
export const HINTS = Object.freeze({
  gamepad: Object.freeze([
    Object.freeze({ glyph: "A", label: "Confirm" }),
    Object.freeze({ glyph: "B", label: "Cancel" }),
    Object.freeze({ glyph: "L-Stick / D-Pad", label: "Move cursor" }),
    Object.freeze({ glyph: "Start", label: "Show or hide hints" }),
  ]),
  keyboard: Object.freeze([
    Object.freeze({ glyph: "Enter", label: "Confirm" }),
    Object.freeze({ glyph: "Esc", label: "Cancel" }),
    Object.freeze({ glyph: "Arrows / WASD", label: "Move cursor" }),
    Object.freeze({ glyph: "H", label: "Show or hide hints" }),
  ]),
});

function defaultGetPads() {
  return typeof navigator !== "undefined" && typeof navigator.getGamepads === "function"
    ? navigator.getGamepads()
    : [];
}
