use std::error::Error;
use std::fmt;

/// Why [`Game::play`] refused a move. A refused move leaves the game unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveError {
    /// The index is not a cell on the board. Valid indexes are 0 to 8.
    OutOfBounds,
    /// The cell already holds a mark.
    Occupied,
    /// The game has already been won or drawn.
    GameOver,
}

impl fmt::Display for MoveError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl Error for MoveError {}
