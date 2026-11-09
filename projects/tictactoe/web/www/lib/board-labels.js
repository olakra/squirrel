// Maps the game state from the wasm wrapper to accessible labels and messages.
// Pure functions. The board is the 9-character string from TicTacToe.board(),
// for example "X...O...X", and status is one of "in_progress", "won_x", "won_o", "draw".

const BOARD_SIZE = 9;
const BOARD_SIDE = 3;
const EMPTY = ".";
const VALID_SYMBOLS = /^[XO.]{9}$/;

// Every line that wins: three rows, three columns, two diagonals (cell indexes).
const WIN_LINES = [
  [0, 1, 2],
  [3, 4, 5],
  [6, 7, 8],
  [0, 3, 6],
  [1, 4, 7],
  [2, 5, 8],
  [0, 4, 8],
  [2, 4, 6],
];

// Maps a status name from the wasm wrapper to the mark that won, if any.
const WINNER_BY_STATUS = { won_x: "X", won_o: "O" };

function assertBoard(board) {
  if (typeof board !== "string" || !VALID_SYMBOLS.test(board)) {
    throw new Error(`invalid board: ${JSON.stringify(board)}`);
  }
}

/** Row and column for a cell index, both counted from 1 for people reading labels. */
function cellPosition(index) {
  return {
    row: Math.floor(index / BOARD_SIDE) + 1,
    column: (index % BOARD_SIDE) + 1,
  };
}

/** One accessible name per cell, for example "Row 1, column 2, empty". */
export function boardLabels(board, t) {
  assertBoard(board);
  return [...board].map((symbol, index) => {
    const state = symbol === EMPTY ? t("cell.state.empty") : t(`mark.${symbol}`);
    return t("cell.label", { ...cellPosition(index), state });
  });
}

/** The three cell indexes of a complete winning line, or null if no line is complete. */
export function winningCells(board) {
  assertBoard(board);
  for (const line of WIN_LINES) {
    const [a, b, c] = line;
    if (board[a] !== EMPTY && board[a] === board[b] && board[a] === board[c]) {
      return line;
    }
  }
  return null;
}

/**
 * Where to draw the win line, in board units from 0 to 3. Each cell centre is at
 * column + 0.5 and row + 0.5, so the line runs from the centre of the first cell
 * to the centre of the last.
 */
export function lineGeometry(line) {
  const first = cellPosition(line[0]);
  const last = cellPosition(line[2]);
  return {
    x1: first.column - 0.5,
    y1: first.row - 0.5,
    x2: last.column - 0.5,
    y2: last.row - 0.5,
  };
}

/** The status message for the live region, such as "X to play" or "O wins". */
export function statusMessage(status, turn, t) {
  if (status === "in_progress") {
    return t("status.turn", { mark: t(`mark.${turn}`) });
  }
  if (status === "draw") {
    return t("status.draw");
  }
  const winner = WINNER_BY_STATUS[status];
  if (winner === undefined) {
    throw new Error(`unknown status: ${status}`);
  }
  return t("status.won", { mark: t(`mark.${winner}`) });
}

/**
 * The message key for a move the game refused. The wasm play() returns only
 * true or false, so the page works out the reason from the state it can see.
 */
export function rejectionKey(board, status) {
  assertBoard(board);
  return status === "in_progress" ? "move.taken" : "move.over";
}
