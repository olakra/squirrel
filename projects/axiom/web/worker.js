// Simulation + render worker. Owns the WASM world, the fixed 60 Hz tick loop
// and the OffscreenCanvas. The main thread never runs simulation code; it only
// sends intents here.
//
// Messages in (main -> worker):
//   { kind: "init", threads }          boot; logs readiness
//   { kind: "canvas", canvas, size }   OffscreenCanvas, transferred once
//   { kind: "resize", size }           new canvas size in device pixels
//   { kind: "intent", intent }         intent.type: "pause" | "resume" | "feed" (item)
//                                      other intent types are ignored here
// Messages out (worker -> main):
//   { kind: "log", msg }               status line text
//
// Renderer: web/renderer.js is optional. If it exists and exports createRenderer,
// the worker draws through it. Otherwise the 2D placeholder is drawn.

// The renderer module is loaded once at startup. A missing file resolves to null.
const rendererReady = import("./renderer.js").catch(() => null);

const TICK_HZ = 60;
const TICK_MS = 1000 / TICK_HZ;
const FRAME_MS = 16; // timer cadence for the loop; ticks are counted by wall clock
const MAX_CATCH_UP_MS = 250; // after a stall, do not run hundreds of ticks at once

// Lane layout passed to axiom_init. Matches the layout used in src/wasm.rs tests.
const LANES = Object.freeze({ count: 2, length: 40, speed: 2 });
const FEED_ITEM_DEFAULT = 1;

// Exports the rest of this file calls. A module missing any of them is rejected.
const REQUIRED_EXPORTS = [
  "axiom_init",
  "axiom_step",
  "axiom_feed",
  "axiom_state_hash_lo",
  "axiom_state_hash_hi",
  "axiom_sunk",
  "axiom_tick_count",
];

let sim = null; // exports of the instantiated axiom.wasm, or null
let renderer = null; // object from createRenderer, or null
let ctx = null; // 2D context, used only when there is no renderer
let canvas = null;
let size = { width: 0, height: 0 };
let paused = false;
let placeholderTicks = 0; // drives the placeholder bar when there is no sim
const instances = new Float32Array(0); // see draw(): no item export yet

const log = (msg) => self.postMessage({ kind: "log", msg });

// ---- WASM loading ----

async function loadSim() {
  const url = new URL("./axiom.wasm", import.meta.url);
  let res;
  try {
    res = await fetch(url, { cache: "no-store" });
  } catch {
    log("axiom.wasm check failed: simulation not loaded, placeholder view only");
    return null;
  }
  if (!res.ok) {
    log(`axiom.wasm not found (HTTP ${res.status}): simulation not loaded, placeholder view only. Run scripts/build-wasm.sh`);
    return null;
  }

  let exports;
  try {
    // Streaming needs application/wasm. The clone keeps the body readable for the fallback.
    const { instance } = await WebAssembly.instantiateStreaming(res.clone(), {});
    exports = instance.exports;
  } catch {
    try {
      const { instance } = await WebAssembly.instantiate(await res.arrayBuffer(), {});
      exports = instance.exports;
    } catch (err) {
      log(`axiom.wasm failed to load: ${err.message}. Placeholder view only`);
      return null;
    }
  }

  const missing = REQUIRED_EXPORTS.filter((name) => typeof exports[name] !== "function");
  if (missing.length > 0) {
    log(`axiom.wasm is missing exports (${missing.join(", ")}): placeholder view only`);
    return null;
  }

  exports.axiom_init(LANES.count, LANES.length, LANES.speed);
  log(`simulation loaded: ${LANES.count} lanes, ${TICK_HZ} Hz`);
  return exports;
}

// Loads in the background. Until it resolves the loop draws the placeholder.
loadSim().then((exports) => {
  sim = exports;
});

// ---- Canvas and renderer ----

function applySize() {
  if (!canvas) return;
  canvas.width = size.width;
  canvas.height = size.height;
  if (renderer) renderer.resize(size.width, size.height);
}

async function attachCanvas(msg) {
  canvas = msg.canvas;
  size = msg.size;
  applySize();

  // Decide the context type before anything calls getContext: a 2D context
  // taken first would stop a WebGL renderer from using this canvas.
  const mod = await rendererReady;
  if (mod && typeof mod.createRenderer === "function") {
    try {
      const r = mod.createRenderer(canvas, { width: size.width, height: size.height });
      if (r && typeof r.draw === "function" && typeof r.resize === "function") {
        renderer = r;
        log("renderer active");
        return;
      }
      log("renderer.js does not return { resize, draw }: using placeholder");
    } catch (err) {
      log(`renderer failed to start: ${err.message}. Using placeholder`);
    }
  }
  ctx = canvas.getContext("2d");
}

// ---- Intents from the main thread ----

function handleIntent(intent) {
  switch (intent.type) {
    case "pause":
      paused = true;
      log("paused");
      break;
    case "resume":
      paused = false;
      log("running");
      break;
    case "feed": {
      if (!sim) {
        log("simulation not loaded: feed ignored");
        break;
      }
      const item = Number.isInteger(intent.item) ? intent.item : FEED_ITEM_DEFAULT;
      const accepted = sim.axiom_feed(item) === 1;
      log(`item ${item} ${accepted ? "accepted" : "rejected"} at lane head`);
      break;
    }
    default:
      break; // UI intents (move, confirm, cancel, toggleHints) stay on the main thread
  }
}

self.onmessage = (e) => {
  const msg = e.data;
  switch (msg.kind) {
    case "init":
      log(`worker ready (threads=${msg.threads})`);
      break;
    case "canvas":
      attachCanvas(msg);
      break;
    case "resize":
      size = msg.size;
      applySize();
      break;
    case "intent":
      handleIntent(msg.intent);
      break;
  }
};

// ---- Fixed-timestep loop ----

function advance(ticks) {
  if (!sim) {
    placeholderTicks += ticks;
    return;
  }
  try {
    sim.axiom_step(ticks);
  } catch (err) {
    // A wasm trap leaves the world in an unknown state. Stop stepping and fall back.
    sim = null;
    paused = true;
    log(`simulation stopped after a trap: ${err.message}`);
  }
}

function draw() {
  if (renderer) {
    try {
      // No item export exists yet: the layer exposes sunk, tick and hash, not item
      // positions. So the instance buffer is empty and the renderer draws no items.
      renderer.draw(instances, 0);
    } catch (err) {
      renderer = null;
      log(`renderer failed: ${err.message}. Using placeholder`);
      ctx = canvas ? canvas.getContext("2d") : null;
    }
    return;
  }
  if (!ctx) return;
  const tick = sim ? sim.axiom_tick_count() : placeholderTicks;
  ctx.fillStyle = "#1E222A";
  ctx.fillRect(0, 0, size.width, size.height);
  ctx.fillStyle = "#FFB81C"; // industrial yellow, per palette
  ctx.fillRect(16, 16, 8 + (tick % 200), 8);
}

let last = performance.now();
let acc = 0;

function loop() {
  const now = performance.now();
  acc += Math.min(now - last, MAX_CATCH_UP_MS);
  last = now;

  if (paused) {
    acc = 0; // resume must not replay the paused time as a burst of ticks
  } else {
    let ticks = 0;
    while (acc >= TICK_MS) {
      acc -= TICK_MS;
      ticks++;
    }
    if (ticks > 0) advance(ticks);
  }

  draw();
  setTimeout(loop, FRAME_MS);
}

setTimeout(loop, FRAME_MS);
