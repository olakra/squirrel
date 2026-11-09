//! Behaviour tests for the public API. They use only what a caller can reach.

use tictactoe_core::{Board, Game, Mark, MoveError, Status};

/// Plays each index in order. Panics if any move is rejected.
fn play_all(game: &mut Game, moves: &[usize]) {
    for &index in moves {
        game.play(index).expect("move should be legal");
    }
}

/// Plays `moves` and checks that the last move wins for `winner` along `line`.
/// The game must still be in progress before the last move.
fn assert_win_on_last_move(moves: &[usize], winner: Mark, line: [usize; 3]) {
    let (last, before) = moves.split_last().expect("a winning game needs moves");
    let mut game = Game::new();
    play_all(&mut game, before);
    assert_eq!(
        game.status(),
        Status::InProgress,
        "{winner:?} on line {line:?} ended the game early"
    );
    game.play(*last).expect("final move should be legal");
    assert_eq!(
        game.status(),
        Status::Won(winner),
        "{winner:?} on line {line:?}"
    );
    for cell in line {
        assert_eq!(
            game.board()[cell],
            Some(winner),
            "cell {cell} of line {line:?}"
        );
    }
}

/// One row per winning line. `x_wins` completes the line for X on X's third move.
/// `o_wins` completes the same line for O on O's third move. The filler moves never
/// complete a line for the other mark.
struct LineCase {
    line: [usize; 3],
    x_wins: &'static [usize],
    o_wins: &'static [usize],
}

const LINE_CASES: [LineCase; 8] = [
    LineCase {
        line: [0, 1, 2],
        x_wins: &[0, 3, 1, 4, 2],
        o_wins: &[3, 0, 4, 1, 6, 2],
    },
    LineCase {
        line: [3, 4, 5],
        x_wins: &[3, 0, 4, 1, 5],
        o_wins: &[0, 3, 1, 4, 6, 5],
    },
    LineCase {
        line: [6, 7, 8],
        x_wins: &[6, 0, 7, 1, 8],
        o_wins: &[0, 6, 1, 7, 3, 8],
    },
    LineCase {
        line: [0, 3, 6],
        x_wins: &[0, 1, 3, 2, 6],
        o_wins: &[1, 0, 2, 3, 4, 6],
    },
    LineCase {
        line: [1, 4, 7],
        x_wins: &[1, 0, 4, 2, 7],
        o_wins: &[0, 1, 2, 4, 3, 7],
    },
    LineCase {
        line: [2, 5, 8],
        x_wins: &[2, 0, 5, 1, 8],
        o_wins: &[0, 2, 1, 5, 3, 8],
    },
    LineCase {
        line: [0, 4, 8],
        x_wins: &[0, 1, 4, 2, 8],
        o_wins: &[1, 0, 2, 4, 3, 8],
    },
    LineCase {
        line: [2, 4, 6],
        x_wins: &[2, 0, 4, 1, 6],
        o_wins: &[0, 2, 1, 4, 3, 6],
    },
];

/// Fills the board with no line, ending as X O X / X O O / O X X.
const DRAW_MOVES: [usize; 9] = [0, 1, 2, 4, 3, 5, 7, 6, 8];

#[test]
fn new_game_has_empty_board_and_x_to_move() {
    let game = Game::new();
    assert_eq!(*game.board(), [None; 9]);
    assert_eq!(game.turn(), Mark::X);
    assert_eq!(game.status(), Status::InProgress);
}

#[test]
fn default_is_the_same_as_new() {
    assert_eq!(Game::default(), Game::new());
}

#[test]
fn turns_alternate_between_x_and_o() {
    let mut game = Game::new();
    let expected = [Mark::O, Mark::X, Mark::O, Mark::X];
    for (index, next) in [0, 1, 2, 3].into_iter().zip(expected) {
        game.play(index).expect("move should be legal");
        assert_eq!(game.turn(), next, "turn after playing {index}");
    }
}

#[test]
fn every_winning_line_wins_for_x() {
    for case in &LINE_CASES {
        assert_win_on_last_move(case.x_wins, Mark::X, case.line);
    }
}

#[test]
fn every_winning_line_wins_for_o() {
    for case in &LINE_CASES {
        assert_win_on_last_move(case.o_wins, Mark::O, case.line);
    }
}

#[test]
fn full_board_without_a_line_is_a_draw() {
    let mut game = Game::new();
    play_all(&mut game, &DRAW_MOVES[..8]);
    assert_eq!(game.status(), Status::InProgress, "one empty cell is left");
    game.play(DRAW_MOVES[8]).expect("last move should be legal");
    assert_eq!(game.status(), Status::Draw);
}

#[test]
fn index_past_the_last_cell_is_out_of_bounds() {
    let mut game = Game::new();
    assert_eq!(game.play(9), Err(MoveError::OutOfBounds));
    assert_eq!(game.play(usize::MAX), Err(MoveError::OutOfBounds));
}

#[test]
fn out_of_bounds_move_changes_nothing() {
    let mut game = Game::new();
    game.play(4).expect("move should be legal");
    let before = game.clone();
    let _ = game.play(9);
    assert_eq!(game, before);
}

#[test]
fn playing_an_occupied_cell_is_rejected() {
    let mut game = Game::new();
    game.play(0).expect("first move should be legal");
    assert_eq!(game.play(0), Err(MoveError::Occupied));
}

#[test]
fn occupied_cell_error_keeps_the_turn_and_board() {
    let mut game = Game::new();
    game.play(0).expect("first move should be legal");
    let board_before: Board = *game.board();
    let _ = game.play(0);
    assert_eq!(
        game.turn(),
        Mark::O,
        "a rejected move must not pass the turn"
    );
    assert_eq!(*game.board(), board_before);
}

#[test]
fn move_after_a_win_is_rejected() {
    let mut game = Game::new();
    play_all(&mut game, &[0, 3, 1, 4, 2]);
    assert_eq!(game.status(), Status::Won(Mark::X));
    let board_before: Board = *game.board();
    assert_eq!(game.play(8), Err(MoveError::GameOver));
    assert_eq!(game.status(), Status::Won(Mark::X));
    assert_eq!(*game.board(), board_before);
}

#[test]
fn move_on_an_occupied_cell_after_a_win_reports_game_over() {
    let mut game = Game::new();
    play_all(&mut game, &[0, 3, 1, 4, 2]);
    assert_eq!(game.play(0), Err(MoveError::GameOver));
}

#[test]
fn out_of_bounds_move_after_a_win_reports_out_of_bounds() {
    let mut game = Game::new();
    play_all(&mut game, &[0, 3, 1, 4, 2]);
    assert_eq!(game.play(9), Err(MoveError::OutOfBounds));
}

#[test]
fn move_after_a_draw_is_rejected() {
    let mut game = Game::new();
    play_all(&mut game, &DRAW_MOVES);
    assert_eq!(game.status(), Status::Draw);
    assert_eq!(game.play(0), Err(MoveError::GameOver));
}

#[test]
fn played_move_changes_only_the_target_cell() {
    let mut game = Game::new();
    for index in [4, 0, 8, 2, 6] {
        let before: Board = *game.board();
        let mover = game.turn();
        game.play(index).expect("move should be legal");
        let after = *game.board();
        for cell in 0..9 {
            if cell == index {
                assert_eq!(after[cell], Some(mover), "target cell {cell}");
            } else {
                assert_eq!(after[cell], before[cell], "untouched cell {cell}");
            }
        }
    }
}

#[test]
fn reset_clears_the_board_and_gives_x_the_turn() {
    let mut game = Game::new();
    play_all(&mut game, &[0, 3, 1, 4, 2]);
    game.reset();
    assert_eq!(game, Game::new());
    assert_eq!(game.status(), Status::InProgress);
}

#[test]
fn reset_allows_play_again() {
    let mut game = Game::new();
    play_all(&mut game, &[0, 3, 1, 4, 2]);
    game.reset();
    assert_eq!(game.play(0), Ok(()));
    assert_eq!(game.board()[0], Some(Mark::X));
}

#[test]
fn move_errors_have_plain_english_messages() {
    assert_eq!(
        MoveError::OutOfBounds.to_string(),
        "that cell is not on the board; use a number from 0 to 8"
    );
    assert_eq!(
        MoveError::Occupied.to_string(),
        "that cell is already taken"
    );
    assert_eq!(MoveError::GameOver.to_string(), "the game is already over");
}

#[test]
fn move_error_is_a_std_error() {
    fn assert_error<E: std::error::Error>(_: &E) {}
    assert_error(&MoveError::Occupied);
}
