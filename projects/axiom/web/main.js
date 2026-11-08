// Main-thread harness. Responsibilities: feature detection, worker dispatch,
// canvas hand-off via OffscreenCanvas, and resize forwarding. No simulation
// runs here, so the main thread stays free for input and UI.

const statusEl = document.getElementById("status");
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
});

worker.onmessage = (e) => {
  if (e.data.kind === "log") log(e.data.msg);
};
