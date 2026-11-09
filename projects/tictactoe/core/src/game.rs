use crate::{Board, Mark, MoveError, Status};

/// A game of tic-tac-toe: the board and the mark whose turn it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    board: Board,
    turn: Mark,
}

impl Game {
    /// Starts a new game with an empty board. X moves first.
    pub fn new() -> Self {
        todo!()
    }

    /// The current board.
    pub fn board(&self) -> &Board {
        todo!()
    }

    /// The mark that moves next.
    pub fn turn(&self) -> Mark {
        todo!()
    }

    /// The state of the game, read from the board.
    pub fn status(&self) -> Status {
        todo!()
    }

    /// Places the current mark on the cell at `index` (row-major, 0 to 8).
    ///
    /// Checks run in this order: index range, then game over, then occupied cell. On
    /// success the turn passes to the other mark. On error nothing changes.
    pub fn play(&mut self, _index: usize) -> Result<(), MoveError> {
        todo!()
    }

    /// Clears the board and returns to a new game, with X to move.
    pub fn reset(&mut self) {
        todo!()
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}
