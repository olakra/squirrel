// Main-thread harness. Responsibilities: feature detection, worker dispatch,
// canvas hand-off via OffscreenCanvas, resize forwarding, and input.
// No simulation runs here, so the main thread stays free for input and UI.
//
// Input flow: input.js turns gamepad and key state into intents. This file
// applies UI intents (cursor, selection markers, hints) locally and forwards
// game intents to the worker as { kind: "intent", intent }. The worker runs
// pause, resume and feed; it ignores the UI intents and the confirm/cancel markers.

import { createGamepadInput, keyToIntent, HINTS } from "./input.js";

const statusEl = document.getElementById("status");
const wasmNoteEl = document.getElementById("wasm-note");
const cursorEl = document.getElementById("cursor");
const markersEl = document.getElementById("markers");
const hintsEl = document.getElementById("hints");
const hintListEl = document.getElementById("hint-list");

const log = (msg) => {
  console.log(`[axiom] ${msg}`);
  statusEl.textContent = msg;
};

// Threads need cross-origin isolation (COOP/COEP) for SharedArrayBuffer.
const canThreads = self.crossOriginIsolated === true && typeof SharedArrayBuffer !== "undefined";
if (!canThreads) {
  log("SharedArrayBuffer unavailable: single-threaded fallback (serve with COOP/COEP to enable threads)");
}
if (typeof OffscreenCanvas === "undefined") {
  log("OffscreenCanvas unsupported: rendering stays on main thread (not implemented yet)");
}

const canvas = document.getElementById("view");
const worker = new Worker(new URL("./worker.js", import.meta.url), { type: "module" });

const dpr = () => Math.min(window.devicePixelRatio || 1, 2);
const sizeOf = () => ({ width: Math.floor(canvas.clientWidth * dpr()), height: Math.floor(canvas.clientHeight * dpr()) });

worker.postMessage({ kind: "init", threads: canThreads });

if (typeof canvas.transferControlToOffscreen === "function") {
  const offscreen = canvas.transferControlToOffscreen();
  worker.postMessage({ kind: "canvas", canvas: offscreen, size: sizeOf() }, [offscreen]);
}

window.addEventListener("resize", () => {
  worker.postMessage({ kind: "resize", size: sizeOf() });
  clampCursor();
  drawCursor();
});

worker.onmessage = (e) => {
  if (e.data.kind === "log") log(e.data.msg);
};
worker.onerror = (e) => log(`worker error: ${e.message || "unknown"}`);

// Game intents go to the worker, which owns the simulation. Keep messages small and plain.
const sendIntent = (intent) => worker.postMessage({ kind: "intent", intent });

// Report a missing WASM module in its own line, so the boot message stays readable.
// Without axiom.wasm the page still runs; the simulation simply is not loaded.
async function probeWasm() {
  try {
    const res = await fetch("./axiom.wasm", { method: "HEAD", cache: "no-store" });
    if (!res.ok) {
      wasmNoteEl.textContent = "axiom.wasm not found: simulation not loaded (run scripts/build-wasm.sh)";
      wasmNoteEl.hidden = false;
    }
  } catch {
    wasmNoteEl.textContent = "axiom.wasm check failed: simulation not loaded";
    wasmNoteEl.hidden = false;
  }
}
probeWasm();

// ---- UI state: cursor and selection markers. Not simulation state. ----

const MAX_MARKERS = 500;
const cursor = { x: 0, y: 0 };
const markers = [];

function clamp(v, lo, hi) {
  return Math.min(hi, Math.max(lo, v));
}

function clampCursor() {
  cursor.x = clamp(cursor.x, 0, window.innerWidth);
  cursor.y = clamp(cursor.y, 0, window.innerHeight);
}

function drawCursor() {
  cursorEl.style.transform = `translate(${cursor.x}px, ${cursor.y}px)`;
}

function addMarker() {
  if (markers.length >= MAX_MARKERS) markers.shift().remove();
  const el = document.createElement("div");
  el.className = "marker";
  el.style.transform = `translate(${cursor.x}px, ${cursor.y}px)`;
  markersEl.appendChild(el);
  markers.push(el);
}

function removeMarker() {
  const el = markers.pop();
  if (el) el.remove();
}

// ---- Hints: one list per device, built with DOM calls (no innerHTML). ----

let device = "keyboard";

function renderHints(next) {
  device = next;
  document.documentElement.dataset.input = device;
  const items = HINTS[device].map(({ glyph, label }) => {
    const li = document.createElement("li");
    li.className = "hint";
    const g = document.createElement("span");
    g.className = "glyph";
    g.textContent = glyph;
    const t = document.createElement("span");
    t.textContent = label;
    li.append(g, t);
    return li;
  });
  hintListEl.replaceChildren(...items);
}

// ---- Intent application ----

function apply(intent, source) {
  if (source !== device) renderHints(source);
  switch (intent.type) {
    case "move":
      cursor.x += intent.dx;
      cursor.y += intent.dy;
      clampCursor();
      drawCursor();
      break;
    case "confirm":
      addMarker();
      sendIntent({ type: "confirm", x: cursor.x, y: cursor.y });
      break;
    case "cancel":
      removeMarker();
      sendIntent({ type: "cancel" });
      break;
    case "toggleHints":
      hintsEl.hidden = !hintsEl.hidden;
      break;
    default:
      break;
  }
}

// ---- Keyboard ----

// Simulation keys. P pauses or resumes the worker's tick loop. F feeds one item to the head of the chain.
// These are not on the hint bar yet.
let paused = false;
window.addEventListener("keydown", (e) => {
  if (!e.repeat && e.code === "KeyP") {
    e.preventDefault();
    paused = !paused;
    sendIntent({ type: paused ? "pause" : "resume" });
    return;
  }
  if (!e.repeat && e.code === "KeyF") {
    e.preventDefault();
    sendIntent({ type: "feed", item: 1 });
    return;
  }
  const intent = keyToIntent(e.code);
  if (!intent) return;
  if (e.repeat && intent.type !== "move") return; // held keys repeat moves only
  e.preventDefault();
  apply(intent, "keyboard");
});

// ---- Gamepad: polled every frame. The Gamepad API is only exposed on the window, not in workers. ----

const pad = createGamepadInput();
window.addEventListener("gamepadconnected", (e) => {
  console.info(`[axiom] controller connected: ${e.gamepad.id}`);
});

function pump(now) {
  const { intents } = pad.poll(now);
  for (const intent of intents) apply(intent, "gamepad");
  requestAnimationFrame(pump);
}

// Start centred, then show keyboard hints until a controller is used.
cursor.x = Math.round(window.innerWidth / 2);
cursor.y = Math.round(window.innerHeight / 2);
drawCursor();
renderHints(device);
requestAnimationFrame(pump);
