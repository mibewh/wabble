use serde::{Deserialize, Serialize};

use crate::tile::Tile;

pub const BOARD_SIZE: usize = 15;
pub const CENTER: usize = 7;

/// Bonus type for a board square.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BonusSquare {
    Normal,
    DoubleLetter,
    TripleLetter,
    DoubleWord,
    TripleWord,
    Center, // also acts as double word
}

/// The 15x15 game board.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    /// Row-major grid of placed tiles. None = empty square.
    cells: [[Option<Tile>; BOARD_SIZE]; BOARD_SIZE],
}

impl Board {
    pub fn new() -> Self {
        Self {
            cells: [[None; BOARD_SIZE]; BOARD_SIZE],
        }
    }

    pub fn get(&self, row: usize, col: usize) -> Option<&Tile> {
        self.cells.get(row)?.get(col)?.as_ref()
    }

    pub fn set(&mut self, row: usize, col: usize, tile: Tile) {
        if row < BOARD_SIZE && col < BOARD_SIZE {
            self.cells[row][col] = Some(tile);
        }
    }

    pub fn is_empty_at(&self, row: usize, col: usize) -> bool {
        row < BOARD_SIZE && col < BOARD_SIZE && self.cells[row][col].is_none()
    }

    pub fn is_occupied(&self, row: usize, col: usize) -> bool {
        row < BOARD_SIZE && col < BOARD_SIZE && self.cells[row][col].is_some()
    }

    /// Whether the board has no tiles placed at all.
    pub fn is_empty(&self) -> bool {
        self.cells.iter().all(|row| row.iter().all(|c| c.is_none()))
    }

    /// Get the bonus type for a square.
    pub fn bonus_at(row: usize, col: usize) -> BonusSquare {
        BONUS_MAP[row][col]
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

/// Standard Scrabble bonus square layout.
/// The board is symmetric along both diagonals, so we define the full 15x15.
const BONUS_MAP: [[BonusSquare; BOARD_SIZE]; BOARD_SIZE] = {
    use BonusSquare::*;
    const N: BonusSquare = Normal;
    const DL: BonusSquare = DoubleLetter;
    const TL: BonusSquare = TripleLetter;
    const DW: BonusSquare = DoubleWord;
    const TW: BonusSquare = TripleWord;
    const CT: BonusSquare = Center;

    [
        [TW, N,  N,  DL, N,  N,  N,  TW, N,  N,  N,  DL, N,  N,  TW],
        [N,  DW, N,  N,  N,  TL, N,  N,  N,  TL, N,  N,  N,  DW, N ],
        [N,  N,  DW, N,  N,  N,  DL, N,  DL, N,  N,  N,  DW, N,  N ],
        [DL, N,  N,  DW, N,  N,  N,  DL, N,  N,  N,  DW, N,  N,  DL],
        [N,  N,  N,  N,  DW, N,  N,  N,  N,  N,  DW, N,  N,  N,  N ],
        [N,  TL, N,  N,  N,  TL, N,  N,  N,  TL, N,  N,  N,  TL, N ],
        [N,  N,  DL, N,  N,  N,  DL, N,  DL, N,  N,  N,  DL, N,  N ],
        [TW, N,  N,  DL, N,  N,  N,  CT, N,  N,  N,  DL, N,  N,  TW],
        [N,  N,  DL, N,  N,  N,  DL, N,  DL, N,  N,  N,  DL, N,  N ],
        [N,  TL, N,  N,  N,  TL, N,  N,  N,  TL, N,  N,  N,  TL, N ],
        [N,  N,  N,  N,  DW, N,  N,  N,  N,  N,  DW, N,  N,  N,  N ],
        [DL, N,  N,  DW, N,  N,  N,  DL, N,  N,  N,  DW, N,  N,  DL],
        [N,  N,  DW, N,  N,  N,  DL, N,  DL, N,  N,  N,  DW, N,  N ],
        [N,  DW, N,  N,  N,  TL, N,  N,  N,  TL, N,  N,  N,  DW, N ],
        [TW, N,  N,  DL, N,  N,  N,  TW, N,  N,  N,  DL, N,  N,  TW],
    ]
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_is_center_bonus() {
        assert_eq!(Board::bonus_at(CENTER, CENTER), BonusSquare::Center);
    }

    #[test]
    fn corners_are_triple_word() {
        assert_eq!(Board::bonus_at(0, 0), BonusSquare::TripleWord);
        assert_eq!(Board::bonus_at(0, 14), BonusSquare::TripleWord);
        assert_eq!(Board::bonus_at(14, 0), BonusSquare::TripleWord);
        assert_eq!(Board::bonus_at(14, 14), BonusSquare::TripleWord);
    }

    #[test]
    fn new_board_is_empty() {
        let board = Board::new();
        assert!(board.is_empty());
    }

    #[test]
    fn set_and_get_tile() {
        let mut board = Board::new();
        let tile = crate::tile::Tile::Letter('A', 1);
        board.set(7, 7, tile);
        assert!(board.is_occupied(7, 7));
        assert_eq!(board.get(7, 7).unwrap().letter(), Some('A'));
    }
}
