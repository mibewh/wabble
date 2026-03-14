use crate::board::{Board, BonusSquare};
use crate::placement::PlacedTile;
use crate::rack::RACK_SIZE;
use crate::tile::Tile;

/// Score a single word formed by a move. `tiles_on_board` are the positions
/// of newly placed tiles (so we know which bonus squares are "fresh").
pub fn score_word(
    word_positions: &[(usize, usize)],
    board: &Board,
    new_tiles: &[PlacedTile],
    placed_tiles_map: &[(usize, usize, Tile)],
) -> i32 {
    let mut word_score = 0i32;
    let mut word_multiplier = 1i32;

    for &(row, col) in word_positions {
        let tile = if let Some(pt) = placed_tiles_map.iter().find(|(r, c, _)| *r == row && *c == col) {
            pt.2
        } else if let Some(t) = board.get(row, col) {
            *t
        } else {
            continue;
        };

        let is_new = new_tiles.iter().any(|nt| nt.row == row && nt.col == col);
        let bonus = if is_new {
            Board::bonus_at(row, col)
        } else {
            BonusSquare::Normal
        };

        let letter_value = tile.points() as i32;
        let letter_mult = match bonus {
            BonusSquare::DoubleLetter => 2,
            BonusSquare::TripleLetter => 3,
            _ => 1,
        };

        word_score += letter_value * letter_mult;

        match bonus {
            BonusSquare::DoubleWord | BonusSquare::Center => word_multiplier *= 2,
            BonusSquare::TripleWord => word_multiplier *= 3,
            _ => {}
        }
    }

    word_score * word_multiplier
}

/// Calculate the bingo bonus (50 points for using all 7 tiles).
pub fn bingo_bonus(tiles_placed: usize) -> i32 {
    if tiles_placed == RACK_SIZE {
        50
    } else {
        0
    }
}

/// Calculate the end-game rack penalty for a player (sum of remaining tile values).
pub fn rack_penalty(tiles: &[Tile]) -> i32 {
    tiles.iter().map(|t| t.points() as i32).sum()
}
