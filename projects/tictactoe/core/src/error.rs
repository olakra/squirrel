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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            MoveError::OutOfBounds => "that cell is not on the board; use a number from 0 to 8",
            MoveError::Occupied => "that cell is already taken",
            MoveError::GameOver => "the game is already over",
        };
        f.write_str(message)
    }
}

impl Error for MoveError {}
