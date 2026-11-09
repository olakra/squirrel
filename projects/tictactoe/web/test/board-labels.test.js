import { test } from "node:test";
import assert from "node:assert/strict";
import {
  boardLabels,
  lineGeometry,
  rejectionKey,
  statusMessage,
  winningCells,
} from "../www/lib/board-labels.js";
import { translator } from "../www/lib/i18n.js";

// English messages, the same keys that www/i18n/en.json defines.
const t = translator({
  "cell.label": "Row {row}, column {column}, {state}",
  "cell.state.empty": "empty",
  "mark.X": "X",
  "mark.O": "O",
  "status.turn": "{mark} to play",
  "status.won": "{mark} wins",
  "status.draw": "It is a draw",
  "move.taken": "That cell is already taken",
  "move.over": "This round is over",
});

const EMPTY_BOARD = ".........";

test("an empty board labels every cell as empty with its row and column", () => {
  const labels = boardLabels(EMPTY_BOARD, t);
  assert.equal(labels.length, 9);
  assert.equal(labels[0], "Row 1, column 1, empty");
  assert.equal(labels[4], "Row 2, column 2, empty");
  assert.equal(labels[8], "Row 3, column 3, empty");
});

test("an occupied cell names its mark instead of empty", () => {
  const labels = boardLabels("X...O....", t);
  assert.equal(labels[0], "Row 1, column 1, X");
  assert.equal(labels[4], "Row 2, column 2, O");
});

test("a malformed board string is rejected", () => {
  assert.throws(() => boardLabels("XX", t), /board/);
  assert.throws(() => boardLabels("Z........", t), /board/);
});

test("winningCells finds a complete line and returns its indexes", () => {
  assert.deepEqual(winningCells("XXX.O.O.."), [0, 1, 2]);
  assert.deepEqual(winningCells("O.X.X.X.O"), [2, 4, 6]);
});

test("winningCells returns null when no line is complete", () => {
  assert.equal(winningCells(EMPTY_BOARD), null);
  assert.equal(winningCells("XOXXOOOXX"), null);
});

test("lineGeometry puts a row line through the centres of its cells", () => {
  assert.deepEqual(lineGeometry([0, 1, 2]), { x1: 0.5, y1: 0.5, x2: 2.5, y2: 0.5 });
});

test("lineGeometry runs a diagonal from corner to corner", () => {
  assert.deepEqual(lineGeometry([0, 4, 8]), { x1: 0.5, y1: 0.5, x2: 2.5, y2: 2.5 });
});

test("statusMessage names the player to move while the game is in progress", () => {
  assert.equal(statusMessage("in_progress", "X", t), "X to play");
});

test("statusMessage names the winner", () => {
  assert.equal(statusMessage("won_o", "O", t), "O wins");
});

test("statusMessage announces a draw", () => {
  assert.equal(statusMessage("draw", "X", t), "It is a draw");
});

test("statusMessage rejects an unknown status name", () => {
  assert.throws(() => statusMessage("paused", "X", t), /paused/);
});

test("rejectionKey says a taken cell is taken while the game runs", () => {
  assert.equal(rejectionKey("X........", "in_progress"), "move.taken");
});

test("rejectionKey says the round is over once the game has ended", () => {
  assert.equal(rejectionKey("XXX.O....", "won_x"), "move.over");
});
