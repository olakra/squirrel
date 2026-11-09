// Page controller. It wires the DOM to the TicTacToe wasm class. The rules and
// the string formats are in the core and the pure modules under lib/.

import { loadMessages, translator } from "./lib/i18n.js";
import {
  boardLabels,
  lineGeometry,
  rejectionKey,
  statusMessage,
  winningCells,
} from "./lib/board-labels.js";

// Paths are relative to index.html.
const MESSAGES_URL = "i18n/en.json";
const WASM_GLUE_URL = "./pkg/tictactoe_web.js";

const BOARD_SIZE = 9;
// Matches TicTacToe.board() for an empty cell.
const EMPTY = ".";
// Matches the status name for a round still in play.
const IN_PROGRESS = "in_progress";

const page = {
  board: document.getElementById("board"),
  status: document.getElementById("status"),
  reset: document.getElementById("reset"),
  winLayer: document.getElementById("win-layer"),
  winLine: document.getElementById("win-line"),
};

// Set once the messages load. Every visible string goes through it.
let t = null;
let game = null;
const cells = [];

function applyStaticText() {
  document.documentElement.lang = t("locale.code");
  document.documentElement.dir = t("locale.dir");
  document.title = t("app.title");
  for (const element of document.querySelectorAll("[data-i18n]")) {
    element.textContent = t(element.dataset.i18n);
  }
  for (const element of document.querySelectorAll("[data-i18n-aria-label]")) {
    element.setAttribute("aria-label", t(element.dataset.i18nAriaLabel));
  }
}

// The wasm module is loaded here, not at the top of the file, so a missing build
// shows a message on the page instead of a blank screen.
async function createGame() {
  const wasm = await import(WASM_GLUE_URL);
  await wasm.default();
  return new wasm.TicTacToe();
}

function buildBoard() {
  for (let index = 0; index < BOARD_SIZE; index += 1) {
    const cell = document.createElement("button");
    cell.type = "button";
    cell.className = "cell";
    cell.dataset.index = String(index);
    cell.addEventListener("click", () => onCellClick(index));
    page.board.appendChild(cell);
    cells.push(cell);
  }
}

// Writes one cell's mark, label and state. Only the attributes change, so the
// buttons stay in place and keep keyboard focus.
function updateCell(cell, symbol, label, roundOver) {
  cell.dataset.state = symbol;
  cell.textContent = symbol === EMPTY ? "" : t(`mark.${symbol}`);
  cell.setAttribute("aria-label", label);
  // aria-disabled, not disabled: a disabled button drops keyboard focus.
  const locked = symbol !== EMPTY || roundOver;
  cell.setAttribute("aria-disabled", String(locked));
}

function renderWinLine(board) {
  const line = winningCells(board);
  page.winLayer.hidden = line === null;
  if (line === null) {
    return;
  }
  const { x1, y1, x2, y2 } = lineGeometry(line);
  page.winLine.setAttribute("x1", String(x1));
  page.winLine.setAttribute("y1", String(y1));
  page.winLine.setAttribute("x2", String(x2));
  page.winLine.setAttribute("y2", String(y2));
}

function render() {
  const board = game.board();
  const status = game.status();
  const labels = boardLabels(board, t);
  const roundOver = status !== IN_PROGRESS;
  cells.forEach((cell, index) => {
    updateCell(cell, board[index], labels[index], roundOver);
  });
  renderWinLine(board);
  page.status.textContent = statusMessage(status, game.turn(), t);
}

// A refused move still reaches the live region, so a screen reader hears why.
function onCellClick(index) {
  const board = game.board();
  const status = game.status();
  if (!game.play(index)) {
    page.status.textContent = t(rejectionKey(board, status));
    return;
  }
  render();
}

function onReset() {
  game.reset();
  render();
}

async function start() {
  const messages = await loadMessages(fetch, MESSAGES_URL);
  t = translator(messages);
  applyStaticText();
  try {
    game = await createGame();
  } catch (error) {
    console.error(error);
    page.status.textContent = t("error.load");
    return;
  }
  buildBoard();
  page.reset.addEventListener("click", onReset);
  render();
}

start().catch((error) => console.error(error));
