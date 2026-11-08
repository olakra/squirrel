// Instanced 2D sprite renderer (WebGL2, GLSL ES 3.00).
//
// Designed to run inside a Web Worker on an OffscreenCanvas, so this module
// never touches the DOM or `window` at import time. The pure helpers at the
// top are exercised by renderer.test.js; the GL path is not testable without
// a browser.
//
// Draw path: one drawArraysInstanced call per draw(). The vertex shader
// expands a shared unit quad (4 vertices, triangle strip) by each instance's
// rect. Instance data is 8 floats per item: x, y, w, h, r, g, b, a.
//
// Buffer strategy: one static VBO for the unit quad, and one dynamic VBO for
// instances. Each draw() copies the caller's instances into a preallocated
// staging Float32Array and uploads only the used prefix with bufferSubData.
// The instance buffer is reallocated only when the count exceeds capacity,
// which doubles it, so steady-state frames do no allocation. A ring of buffers
// was considered, but a single buffer is enough for one worker-side draw per
// frame. Whether it stalls on real drivers is unverified.
//
// Context loss: the webglcontextlost handler calls preventDefault, which is
// required for the browser to send webglcontextrestored. On restore, every GL
// object is recreated. The CPU-side staging array is kept.

// ---- Pure helpers (no GL, no DOM). Exercised by renderer.test.js. ----

export const FLOATS_PER_INSTANCE = 8;
export const BYTES_PER_INSTANCE = FLOATS_PER_INSTANCE * 4;
const INITIAL_CAPACITY = 4096; // instances

// Shape ids for item classes. The shader does not use them yet, but they are
// part of the palette so the item-class encoding is fixed before the sprite
// artwork lands. Colour alone must not carry item identity.
export const SHAPE = Object.freeze({ SQUARE: 0, DIAMOND: 1, CIRCLE: 2 });

const DEFAULT_BACKGROUND_HEX = "#1E222A"; // factory floor

// Writes one instance (8 floats) at `index` and returns the buffer.
export function packInstance(buffer, index, rect, colour) {
  const o = index * FLOATS_PER_INSTANCE;
  buffer[o] = rect.x;
  buffer[o + 1] = rect.y;
  buffer[o + 2] = rect.w;
  buffer[o + 3] = rect.h;
  buffer[o + 4] = colour.r;
  buffer[o + 5] = colour.g;
  buffer[o + 6] = colour.b;
  buffer[o + 7] = colour.a;
  return buffer;
}

// Returns an integer count in [0, capacity]. Non-finite or negative counts
// become 0, and fractions are floored.
export function clampCount(count, capacity) {
  const cap = Number.isFinite(capacity) && capacity > 0 ? Math.floor(capacity) : 0;
  if (!Number.isFinite(count) || count <= 0) return 0;
  return Math.min(Math.floor(count), cap);
}

// Parses "#RRGGBB" into 0..1 floats. Returns null for anything else.
export function parseHex(hex) {
  if (typeof hex !== "string") return null;
  const m = /^#([0-9a-fA-F]{6})$/.exec(hex);
  if (!m) return null;
  const n = parseInt(m[1], 16);
  return Object.freeze({
    r: ((n >> 16) & 255) / 255,
    g: ((n >> 8) & 255) / 255,
    b: (n & 255) / 255,
    a: 1,
  });
}

const clamp01 = (v) => (Number.isFinite(v) ? Math.min(1, Math.max(0, v)) : 0);

// Accepts "#RRGGBB" or [r, g, b] / [r, g, b, a] in 0..1. Anything else gives
// the factory-floor default.
export function parseBackground(value) {
  if (Array.isArray(value) && value.length >= 3) {
    return Object.freeze({
      r: clamp01(value[0]),
      g: clamp01(value[1]),
      b: clamp01(value[2]),
      a: value.length > 3 ? clamp01(value[3]) : 1,
    });
  }
  return parseHex(value) ?? parseHex(DEFAULT_BACKGROUND_HEX);
}

// Palette (fixed table). Hex values follow the project palette. Each item
// class has a distinct shape id, and colours differ in lightness as well as
// hue. Solids are squares and diamonds. Fluids are circles.
const ITEM_CLASSES = Object.freeze([
  { hex: "#FFB81C", shape: SHAPE.SQUARE }, // warm yellow: belt items
  { hex: "#FF6A13", shape: SHAPE.DIAMOND }, // safety orange: components
  { hex: "#2EE6F0", shape: SHAPE.SQUARE }, // cyan highlight: circuits
  { hex: "#7EC8F0", shape: SHAPE.CIRCLE }, // clear blue: water
  { hex: "#4A6B1F", shape: SHAPE.CIRCLE }, // dark sludge green: heavy fuel
  { hex: "#B6FF2E", shape: SHAPE.CIRCLE }, // vivid lime: acid
].map((c) => Object.freeze({ ...c, colour: parseHex(c.hex) })));

// Maps an item id to a class. Non-finite ids map to class 0. Negative ids wrap.
function itemClass(itemId) {
  const id = Number.isFinite(itemId) ? Math.trunc(itemId) : 0;
  const n = ITEM_CLASSES.length;
  return ITEM_CLASSES[((id % n) + n) % n];
}

// Deterministic: the same item id always gives the same colour.
export function colourForItem(itemId) {
  return itemClass(itemId).colour;
}

// Shape id in 0..2. Deterministic, and it does not depend on colour.
export function shapeForItem(itemId) {
  return itemClass(itemId).shape;
}

// ---- GL path. Nothing below runs until createRenderer is called. ----

const VERTEX_SRC = `#version 300 es
layout(location = 0) in vec2 a_corner;   // unit quad, 0..1
layout(location = 1) in vec4 a_rect;     // x, y, w, h in 0..1, origin top-left
layout(location = 2) in vec4 a_colour;   // r, g, b, a
out vec4 v_colour;
void main() {
  vec2 p = a_rect.xy + a_corner * a_rect.zw;
  gl_Position = vec4(p.x * 2.0 - 1.0, 1.0 - p.y * 2.0, 0.0, 1.0);
  v_colour = a_colour;
}
`;

const FRAGMENT_SRC = `#version 300 es
precision mediump float;
in vec4 v_colour;
out vec4 o_colour;
void main() {
  o_colour = v_colour;
}
`;

// Triangle strip: (0,0), (1,0), (0,1), (1,1).
const QUAD = new Float32Array([0, 0, 1, 0, 0, 1, 1, 1]);

const warn = (msg) => console.warn(`[axiom] renderer: ${msg}`);

function compile(gl, type, src) {
  const shader = gl.createShader(type);
  gl.shaderSource(shader, src);
  gl.compileShader(shader);
  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    const info = gl.getShaderInfoLog(shader);
    gl.deleteShader(shader);
    throw new Error(`shader compile failed: ${info}`);
  }
  return shader;
}

// Creates a renderer, or returns null when WebGL2 cannot be used. The caller
// should fall back to the 2D context. Never throws for missing WebGL2.
export function createRenderer(canvas, options = {}) {
  if (!canvas || typeof canvas.getContext !== "function") {
    warn("no canvas given; WebGL2 unavailable, falling back");
    return null;
  }

  const background = parseBackground(options.background);
  const dim = (v, fallback) => (Number.isFinite(v) && v >= 1 ? Math.floor(v) : fallback);
  let width = dim(options.width, Math.max(1, canvas.width | 0));
  let height = dim(options.height, Math.max(1, canvas.height | 0));

  let gl = null;
  try {
    gl = canvas.getContext("webgl2", {
      alpha: false,
      antialias: false,
      depth: false,
      stencil: false,
      preserveDrawingBuffer: false,
    });
  } catch {
    gl = null;
  }
  if (!gl || gl.isContextLost()) {
    warn("WebGL2 unavailable, falling back to the 2D canvas");
    return null;
  }

  // GL handles. Recreated by buildGL after a context restore.
  const res = { program: null, vao: null, quadVbo: null, instanceVbo: null };
  let capacity = INITIAL_CAPACITY;
  let staging = new Float32Array(capacity * FLOATS_PER_INSTANCE);
  let lost = false;
  let disposed = false;

  function buildGL() {
    const vs = compile(gl, gl.VERTEX_SHADER, VERTEX_SRC);
    const fs = compile(gl, gl.FRAGMENT_SHADER, FRAGMENT_SRC);
    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);
    gl.deleteShader(vs);
    gl.deleteShader(fs);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
      const info = gl.getProgramInfoLog(program);
      gl.deleteProgram(program);
      throw new Error(`program link failed: ${info}`);
    }

    const vao = gl.createVertexArray();
    gl.bindVertexArray(vao);

    const quadVbo = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, quadVbo);
    gl.bufferData(gl.ARRAY_BUFFER, QUAD, gl.STATIC_DRAW);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);

    const instanceVbo = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, instanceVbo);
    gl.bufferData(gl.ARRAY_BUFFER, capacity * BYTES_PER_INSTANCE, gl.DYNAMIC_DRAW);
    gl.enableVertexAttribArray(1);
    gl.vertexAttribPointer(1, 4, gl.FLOAT, false, BYTES_PER_INSTANCE, 0);
    gl.vertexAttribDivisor(1, 1);
    gl.enableVertexAttribArray(2);
    gl.vertexAttribPointer(2, 4, gl.FLOAT, false, BYTES_PER_INSTANCE, 16);
    gl.vertexAttribDivisor(2, 1);

    gl.bindVertexArray(null);
    gl.bindBuffer(gl.ARRAY_BUFFER, null);

    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
    gl.viewport(0, 0, width, height);

    res.program = program;
    res.vao = vao;
    res.quadVbo = quadVbo;
    res.instanceVbo = instanceVbo;
  }

  function freeGL() {
    if (gl.isContextLost()) return;
    if (res.program) gl.deleteProgram(res.program);
    if (res.vao) gl.deleteVertexArray(res.vao);
    if (res.quadVbo) gl.deleteBuffer(res.quadVbo);
    if (res.instanceVbo) gl.deleteBuffer(res.instanceVbo);
    res.program = res.vao = res.quadVbo = res.instanceVbo = null;
  }

  try {
    buildGL();
  } catch (err) {
    warn(`${err.message}; falling back to the 2D canvas`);
    return null;
  }

  canvas.width = width;
  canvas.height = height;

  // Grows the instance buffer (doubling) when a draw needs more room. This is
  // rare and never runs on a steady-state frame.
  function ensureCapacity(needed) {
    if (needed <= capacity) return;
    let next = capacity;
    while (next < needed) next *= 2;
    capacity = next;
    staging = new Float32Array(capacity * FLOATS_PER_INSTANCE);
    gl.bindBuffer(gl.ARRAY_BUFFER, res.instanceVbo);
    gl.bufferData(gl.ARRAY_BUFFER, capacity * BYTES_PER_INSTANCE, gl.DYNAMIC_DRAW);
    gl.bindBuffer(gl.ARRAY_BUFFER, null);
  }

  function onLost(event) {
    // Required, or the browser will not send webglcontextrestored.
    event.preventDefault();
    lost = true;
  }

  function onRestored() {
    try {
      buildGL();
      lost = false;
    } catch (err) {
      warn(`restore failed: ${err.message}`);
      lost = true;
    }
  }

  canvas.addEventListener("webglcontextlost", onLost);
  canvas.addEventListener("webglcontextrestored", onRestored);

  return {
    resize(w, h) {
      if (disposed) return;
      width = dim(w, width);
      height = dim(h, height);
      canvas.width = width;
      canvas.height = height;
      if (!lost) gl.viewport(0, 0, width, height);
    },

    // instances: Float32Array with 8 floats per item. Returns false when nothing was drawn.
    draw(instances, count) {
      if (disposed || lost) return false;
      const available = instances ? Math.floor(instances.length / FLOATS_PER_INSTANCE) : 0;
      const n = clampCount(count, available);
      ensureCapacity(n);

      if (n > 0) staging.set(instances.subarray(0, n * FLOATS_PER_INSTANCE), 0);

      gl.clearColor(background.r, background.g, background.b, background.a);
      gl.clear(gl.COLOR_BUFFER_BIT);

      if (n > 0) {
        gl.bindBuffer(gl.ARRAY_BUFFER, res.instanceVbo);
        gl.bufferSubData(gl.ARRAY_BUFFER, 0, staging, 0, n * FLOATS_PER_INSTANCE);
        gl.useProgram(res.program);
        gl.bindVertexArray(res.vao);
        gl.drawArraysInstanced(gl.TRIANGLE_STRIP, 0, 4, n);
        gl.bindVertexArray(null);
      }
      return true;
    },

    dispose() {
      if (disposed) return;
      disposed = true;
      canvas.removeEventListener("webglcontextlost", onLost);
      canvas.removeEventListener("webglcontextrestored", onRestored);
      freeGL();
      staging = null;
    },
  };
}
