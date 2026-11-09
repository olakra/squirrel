use crate::{Board, Mark, MoveError, Status, CELL_COUNT};

/// The eight lines that win the game, as row-major cell indexes.
const WINNING_LINES: [[usize; 3]; 8] = [
    [0, 1, 2], // top row
    [3, 4, 5], // middle row
    [6, 7, 8], // bottom row
    [0, 3, 6], // left column
    [1, 4, 7], // middle column
    [2, 5, 8], // right column
    [0, 4, 8], // main diagonal
    [2, 4, 6], // anti-diagonal
];

/// A game of tic-tac-toe: the board and the mark whose turn it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    board: Board,
    turn: Mark,
}

impl Game {
    /// Starts a new game with an empty board. X moves first.
    pub fn new() -> Self {
        Game {
            board: [None; CELL_COUNT],
            turn: Mark::X,
        }
    }

    /// The current board.
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// The mark that moves next.
    pub fn turn(&self) -> Mark {
        self.turn
    }

    /// The state of the game, read from the board.
    pub fn status(&self) -> Status {
        status_of(&self.board)
    }

    /// Places the current mark on the cell at `index` (row-major, 0 to 8).
    ///
    /// Checks run in this order: index range, then game over, then occupied cell. On
    /// success the turn passes to the other mark. On error nothing changes.
    pub fn play(&mut self, index: usize) -> Result<(), MoveError> {
        self.check_move(index)?;
        self.board[index] = Some(self.turn);
        self.turn = self.turn.other();
        Ok(())
    }

    /// Clears the board and returns to a new game, with X to move.
    pub fn reset(&mut self) {
        *self = Game::new();
    }

    /// Returns the first rule that `index` breaks, or `Ok` if the move is legal.
    fn check_move(&self, index: usize) -> Result<(), MoveError> {
        if index >= CELL_COUNT {
            return Err(MoveError::OutOfBounds);
        }
        if self.status() != Status::InProgress {
            return Err(MoveError::GameOver);
        }
        if self.board[index].is_some() {
            return Err(MoveError::Occupied);
        }
        Ok(())
    }
}

/// Reads the game state from the board alone.
fn status_of(board: &Board) -> Status {
    if let Some(mark) = winner(board) {
        Status::Won(mark)
    } else if board.iter().all(Option::is_some) {
        Status::Draw
    } else {
        Status::InProgress
    }
}

/// The mark that fills a winning line, if one does.
fn winner(board: &Board) -> Option<Mark> {
    WINNING_LINES.iter().find_map(|line| {
        let mark = board[line[0]]?;
        line.iter()
            .all(|&cell| board[cell] == Some(mark))
            .then_some(mark)
    })
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}
