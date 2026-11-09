//! Browser wrapper: exposes the tic-tac-toe core to JavaScript as the `TicTacToe` class.
//!
//! Every method only converts types at the boundary. The rules live in `tictactoe-core`,
//! and the string forms live in `convert`, which is tested on the host.

pub mod convert;

use tictactoe_core::Game;
use wasm_bindgen::prelude::*;

/// One game of tic-tac-toe as seen from JavaScript.
#[wasm_bindgen]
pub struct TicTacToe {
    game: Game,
}

impl Default for TicTacToe {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl TicTacToe {
    /// A new game with an empty board and X to move.
    #[wasm_bindgen(constructor)]
    pub fn new() -> TicTacToe {
        TicTacToe { game: Game::new() }
    }

    /// The 9 cells, row by row from the top-left: "X", "O" or "." for empty.
    pub fn board(&self) -> String {
        convert::board_to_string(self.game.board())
    }

    /// Whose turn it is: "X" or "O".
    pub fn turn(&self) -> String {
        convert::mark_name(self.game.turn()).to_string()
    }

    /// The game state: "in_progress", "won_x", "won_o" or "draw".
    pub fn status(&self) -> String {
        convert::status_name(self.game.status()).to_string()
    }

    /// Plays the cell at `index` (0 to 8). Returns true if the move was applied,
    /// false if it was refused. A refused move changes nothing.
    pub fn play(&mut self, index: usize) -> bool {
        self.game.play(index).is_ok()
    }

    /// Clears the board and gives X the first move.
    pub fn reset(&mut self) {
        self.game.reset();
    }
}
