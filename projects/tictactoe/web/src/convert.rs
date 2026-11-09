//! Pure conversions from core types to the strings the browser reads.
//!
//! This module does not depend on wasm-bindgen, so its tests run on the host
//! with no browser and no WebAssembly target.

use tictactoe_core::{Board, Mark, Status};

/// The character that stands for an empty cell in the board string.
pub const EMPTY_SYMBOL: char = '.';

/// One board cell as a character: "X", "O" or "." for empty.
pub fn cell_symbol(cell: Option<Mark>) -> char {
    match cell {
        None => EMPTY_SYMBOL,
        Some(Mark::X) => 'X',
        Some(Mark::O) => 'O',
    }
}

/// The 9 cells as a 9-character string, row by row from the top-left cell.
pub fn board_to_string(board: &Board) -> String {
    board.iter().map(|&cell| cell_symbol(cell)).collect()
}

/// The name of a mark, "X" or "O".
pub fn mark_name(mark: Mark) -> &'static str {
    match mark {
        Mark::X => "X",
        Mark::O => "O",
    }
}

/// The status as a lowercase name: "in_progress", "won_x", "won_o" or "draw".
///
/// These names are part of the contract with main.js, so change them only together.
pub fn status_name(status: Status) -> &'static str {
    match status {
        Status::InProgress => "in_progress",
        Status::Won(Mark::X) => "won_x",
        Status::Won(Mark::O) => "won_o",
        Status::Draw => "draw",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_cell_is_a_dot() {
        assert_eq!(cell_symbol(None), '.');
    }

    #[test]
    fn marked_cells_are_their_letter() {
        assert_eq!(cell_symbol(Some(Mark::X)), 'X');
        assert_eq!(cell_symbol(Some(Mark::O)), 'O');
    }

    #[test]
    fn empty_board_is_nine_dots() {
        assert_eq!(board_to_string(&[None; 9]), ".........");
    }

    #[test]
    fn board_keeps_row_major_order() {
        let mut board: Board = [None; 9];
        board[0] = Some(Mark::X);
        board[4] = Some(Mark::O);
        board[8] = Some(Mark::X);
        assert_eq!(board_to_string(&board), "X...O...X");
    }

    #[test]
    fn mark_names_are_letters() {
        assert_eq!(mark_name(Mark::X), "X");
        assert_eq!(mark_name(Mark::O), "O");
    }

    #[test]
    fn status_names_are_stable_lowercase_words() {
        assert_eq!(status_name(Status::InProgress), "in_progress");
        assert_eq!(status_name(Status::Won(Mark::X)), "won_x");
        assert_eq!(status_name(Status::Won(Mark::O)), "won_o");
        assert_eq!(status_name(Status::Draw), "draw");
    }
}
