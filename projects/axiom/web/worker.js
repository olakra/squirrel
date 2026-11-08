// Simulation + render worker. The WASM module is not built yet, so this
// worker only proves the wiring: message passing, the OffscreenCanvas
// handoff, and a fixed-timestep loop at 60 Hz.

const TICK_MS = 1000 / 60;
let canvas = null;
let ctx = null;
let size = { width: 0, height: 0 };
let tick = 0;

self.onmessage = (e) => {
  const msg = e.data;
  switch (msg.kind) {
    case "init":
      self.postMessage({ kind: "log", msg: `worker ready (threads=${msg.threads})` });
      break;
    case "canvas":
      canvas = msg.canvas;
      size = msg.size;
      canvas.width = size.width;
      canvas.height = size.height;
      ctx = canvas.getContext("2d");
      break;
    case "resize":
      size = msg.size;
      if (canvas) {
        canvas.width = size.width;
        canvas.height = size.height;
      }
      break;
  }
};

// Fixed timestep: advance the sim by whole ticks using wall-clock
// accumulation, decoupled from the render cadence.
let last = performance.now();
let acc = 0;

function frame(now) {
  acc += now - last;
  last = now;
  while (acc >= TICK_MS) {
    tick++; // placeholder for world.step() once the WASM module is wired in
    acc -= TICK_MS;
  }
  if (ctx) {
    ctx.fillStyle = "#1E222A";
    ctx.fillRect(0, 0, size.width, size.height);
    ctx.fillStyle = "#FFB81C"; // industrial yellow, per palette
    ctx.fillRect(16, 16, 8 + (tick % 200), 8);
  }
  requestAnimationFrame ? requestAnimationFrame(frame) : setTimeout(() => frame(performance.now()), TICK_MS);
}
// Workers may not have requestAnimationFrame in every browser; fall back to timers.
if (typeof requestAnimationFrame === "function") {
  requestAnimationFrame(frame);
} else {
  setTimeout(() => frame(performance.now()), TICK_MS);
}
