// Turns a WASM item snapshot into renderer instances. Pure JavaScript: no DOM
// and no WebGL, so it runs under `bun test`. The worker passes in the
// renderer's packInstance and colourForItem.
//
// Layout: lane 0 is the top row, and each lane runs left to right. Offsets are
// fractions of the lane length, so the same layout works at any canvas size.

// Matches FLOATS_PER_INSTANCE in renderer.js. items.test.js checks the match.
export const FLOATS_PER_INSTANCE = 8;

export const LAYOUT = Object.freeze({
  xStart: 0.1, // left end of a lane, in canvas fractions (origin top-left)
  xEnd: 0.9, // right end of a lane
  firstRowY: 0.3, // centre of lane 0
  rowGap: 0.2, // distance between lane rows
  itemW: 0.03,
  itemH: 0.05,
});

const clamp01 = (v) => Math.min(1, Math.max(0, v));

// Rectangle (top-left origin, 0..1) for an item at `offset` on `lane`.
// Offsets past the lane end are clamped to its end.
export function itemRect(lane, offset, laneLength) {
  const t = laneLength > 0 ? clamp01(offset / laneLength) : 0;
  const cx = LAYOUT.xStart + (LAYOUT.xEnd - LAYOUT.xStart) * t;
  const cy = LAYOUT.firstRowY + LAYOUT.rowGap * lane;
  return {
    x: cx - LAYOUT.itemW / 2,
    y: cy - LAYOUT.itemH / 2,
    w: LAYOUT.itemW,
    h: LAYOUT.itemH,
  };
}

// Writes up to `count` items into `buffer`, which must hold FLOATS_PER_INSTANCE
// floats per item. `read` gives the snapshot fields by index: read.id(i),
// read.lane(i) and read.offset(i). Returns how many items were written, which
// is less than `count` if the buffer is too small.
export function writeInstances(buffer, count, read, laneLength, packInstance, colourForItem) {
  const capacity = Math.floor(buffer.length / FLOATS_PER_INSTANCE);
  const n = Math.min(Math.max(0, Math.floor(count)), capacity);
  for (let i = 0; i < n; i++) {
    const rect = itemRect(read.lane(i), read.offset(i), laneLength);
    packInstance(buffer, i, rect, colourForItem(read.id(i)));
  }
  return n;
}
