//! Pure Rust rules for tic-tac-toe: board, turns, win and draw detection.
//!
//! The crate has no dependencies and performs no I/O. A [`Game`] owns the board and
//! whose turn it is. Each call to [`Game::play`] either applies one legal move or
//! returns a [`MoveError`] and changes nothing.

mod error;
mod game;

pub use error::MoveError;
pub use game::Game;

/// Number of cells on the board: a 3 by 3 grid.
const CELL_COUNT: usize = 9;

/// The board as a flat, row-major array. Index 0 is the top-left cell and index 8 is the
/// bottom-right cell. `None` means the cell is empty.
pub type Board = [Option<Mark>; CELL_COUNT];

/// A player's mark. X always moves first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    /// The player who moves first.
    X,
    /// The player who moves second.
    O,
}

/// The state of a game, as read from the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// No line is complete and at least one cell is empty.
    InProgress,
    /// The given mark has completed a winning line.
    Won(Mark),
    /// The board is full and no line is complete.
    Draw,
}
