use serde::{Deserialize, Serialize};

use crate::board::{BOARD_SIZE, Board, CENTER};
use crate::error::WordGameError;
use crate::tile::Tile;

/// A tile placed on the board by a player during a turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacedTile {
    pub row: usize,
    pub col: usize,
    pub tile: Tile,
}

/// Direction of word placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Horizontal,
    Vertical,
}

/// A validated placement with extracted words.
#[derive(Debug, Clone)]
pub struct ValidatedPlacement {
    pub tiles: Vec<PlacedTile>,
    pub direction: Direction,
    /// All words formed (primary + cross-words), as lists of (row, col) positions.
    pub words_formed: Vec<Vec<(usize, usize)>>,
    /// The actual word strings formed.
    pub word_strings: Vec<String>,
}

/// Validate tile placement on the board.
pub fn validate_placement(
    board: &Board,
    tiles: &[PlacedTile],
    is_first_move: bool,
) -> Result<ValidatedPlacement, WordGameError> {
    if tiles.is_empty() {
        return Err(WordGameError::InvalidPlacement(
            "must place at least one tile".into(),
        ));
    }

    // Check all positions are in bounds and empty
    for t in tiles {
        if t.row >= BOARD_SIZE || t.col >= BOARD_SIZE {
            return Err(WordGameError::InvalidPlacement(format!(
                "position ({}, {}) is out of bounds",
                t.row, t.col
            )));
        }
        if board.is_occupied(t.row, t.col) {
            return Err(WordGameError::InvalidPlacement(format!(
                "position ({}, {}) is already occupied",
                t.row, t.col
            )));
        }
    }

    // Check no duplicate positions
    for i in 0..tiles.len() {
        for j in (i + 1)..tiles.len() {
            if tiles[i].row == tiles[j].row && tiles[i].col == tiles[j].col {
                return Err(WordGameError::InvalidPlacement(
                    "duplicate tile positions".into(),
                ));
            }
        }
    }

    // Determine direction
    let direction = determine_direction(tiles)?;

    // Check tiles are contiguous (with existing board tiles filling gaps)
    check_contiguous(board, tiles, direction)?;

    // First move must cover center
    if is_first_move {
        let covers_center = tiles.iter().any(|t| t.row == CENTER && t.col == CENTER);
        if !covers_center {
            return Err(WordGameError::InvalidPlacement(
                "first move must cover the center square".into(),
            ));
        }
        // First move must form a word of at least 2 letters
        if tiles.len() < 2 {
            return Err(WordGameError::InvalidPlacement(
                "first move must place at least 2 tiles".into(),
            ));
        }
    } else {
        // Must be adjacent to at least one existing tile
        let adjacent = tiles.iter().any(|t| has_adjacent_tile(board, t.row, t.col));
        if !adjacent {
            return Err(WordGameError::InvalidPlacement(
                "tiles must be adjacent to existing tiles".into(),
            ));
        }
    }

    // Extract all words formed
    let (words_formed, word_strings) =
        extract_words(board, tiles, direction)?;

    if words_formed.is_empty() {
        return Err(WordGameError::InvalidPlacement(
            "no words formed".into(),
        ));
    }

    Ok(ValidatedPlacement {
        tiles: tiles.to_vec(),
        direction,
        words_formed,
        word_strings,
    })
}

fn determine_direction(tiles: &[PlacedTile]) -> Result<Direction, WordGameError> {
    if tiles.len() == 1 {
        // Single tile: direction doesn't matter, default to Horizontal
        return Ok(Direction::Horizontal);
    }

    let same_row = tiles.iter().all(|t| t.row == tiles[0].row);
    let same_col = tiles.iter().all(|t| t.col == tiles[0].col);

    match (same_row, same_col) {
        (true, _) => Ok(Direction::Horizontal),
        (_, true) => Ok(Direction::Vertical),
        _ => Err(WordGameError::InvalidPlacement(
            "tiles must be in a single row or column".into(),
        )),
    }
}

fn check_contiguous(
    board: &Board,
    tiles: &[PlacedTile],
    direction: Direction,
) -> Result<(), WordGameError> {
    if tiles.len() <= 1 {
        return Ok(());
    }

    let (main_positions, fixed): (Vec<usize>, usize) = match direction {
        Direction::Horizontal => {
            let row = tiles[0].row;
            (tiles.iter().map(|t| t.col).collect(), row)
        }
        Direction::Vertical => {
            let col = tiles[0].col;
            (tiles.iter().map(|t| t.row).collect(), col)
        }
    };

    let min = *main_positions.iter().min().unwrap();
    let max = *main_positions.iter().max().unwrap();

    // Every position between min and max must be either a new tile or existing board tile
    for pos in min..=max {
        let (row, col) = match direction {
            Direction::Horizontal => (fixed, pos),
            Direction::Vertical => (pos, fixed),
        };
        let is_new = tiles.iter().any(|t| t.row == row && t.col == col);
        let is_existing = board.is_occupied(row, col);
        if !is_new && !is_existing {
            return Err(WordGameError::InvalidPlacement(
                "tiles must form a contiguous line".into(),
            ));
        }
    }

    Ok(())
}

fn has_adjacent_tile(board: &Board, row: usize, col: usize) -> bool {
    let neighbors = [
        (row.wrapping_sub(1), col),
        (row + 1, col),
        (row, col.wrapping_sub(1)),
        (row, col + 1),
    ];
    neighbors
        .iter()
        .any(|&(r, c)| r < BOARD_SIZE && c < BOARD_SIZE && board.is_occupied(r, c))
}

/// Build a temporary board view with newly placed tiles added.
fn tile_at<'a>(board: &'a Board, tiles: &'a [PlacedTile], row: usize, col: usize) -> Option<Tile> {
    if let Some(t) = board.get(row, col) {
        return Some(*t);
    }
    tiles
        .iter()
        .find(|t| t.row == row && t.col == col)
        .map(|t| t.tile)
}

fn tile_letter(board: &Board, tiles: &[PlacedTile], row: usize, col: usize) -> Option<char> {
    tile_at(board, tiles, row, col).and_then(|t| t.letter())
}

/// Positions and string representations of words formed by a placement.
type ExtractedWords = (Vec<Vec<(usize, usize)>>, Vec<String>);

/// Extract all words formed by the placement.
fn extract_words(
    board: &Board,
    tiles: &[PlacedTile],
    direction: Direction,
) -> Result<ExtractedWords, WordGameError> {
    let mut all_positions: Vec<Vec<(usize, usize)>> = Vec::new();
    let mut all_strings: Vec<String> = Vec::new();

    // Primary word along the placement direction
    let primary = extract_word_along(board, tiles, direction);
    if primary.len() >= 2 {
        let s: String = primary
            .iter()
            .filter_map(|&(r, c)| tile_letter(board, tiles, r, c))
            .collect();
        all_positions.push(primary);
        all_strings.push(s);
    }

    // Cross-words perpendicular to placement direction
    let cross_dir = match direction {
        Direction::Horizontal => Direction::Vertical,
        Direction::Vertical => Direction::Horizontal,
    };

    for t in tiles {
        let cross = extract_cross_word(board, tiles, t.row, t.col, cross_dir);
        if cross.len() >= 2 {
            let s: String = cross
                .iter()
                .filter_map(|&(r, c)| tile_letter(board, tiles, r, c))
                .collect();
            all_positions.push(cross);
            all_strings.push(s);
        }
    }

    Ok((all_positions, all_strings))
}

/// Extract the full word along `direction` that includes the placed tiles.
fn extract_word_along(
    board: &Board,
    tiles: &[PlacedTile],
    direction: Direction,
) -> Vec<(usize, usize)> {
    // Find the extent of the word
    let (fixed, min_pos, max_pos) = match direction {
        Direction::Horizontal => {
            let row = tiles[0].row;
            let cols: Vec<usize> = tiles.iter().map(|t| t.col).collect();
            (row, *cols.iter().min().unwrap(), *cols.iter().max().unwrap())
        }
        Direction::Vertical => {
            let col = tiles[0].col;
            let rows: Vec<usize> = tiles.iter().map(|t| t.row).collect();
            (col, *rows.iter().min().unwrap(), *rows.iter().max().unwrap())
        }
    };

    // Extend backward
    let mut start = min_pos;
    while start > 0 {
        let (r, c) = match direction {
            Direction::Horizontal => (fixed, start - 1),
            Direction::Vertical => (start - 1, fixed),
        };
        if tile_at(board, tiles, r, c).is_some() {
            start -= 1;
        } else {
            break;
        }
    }

    // Extend forward
    let mut end = max_pos;
    while end + 1 < BOARD_SIZE {
        let (r, c) = match direction {
            Direction::Horizontal => (fixed, end + 1),
            Direction::Vertical => (end + 1, fixed),
        };
        if tile_at(board, tiles, r, c).is_some() {
            end += 1;
        } else {
            break;
        }
    }

    (start..=end)
        .map(|pos| match direction {
            Direction::Horizontal => (fixed, pos),
            Direction::Vertical => (pos, fixed),
        })
        .collect()
}

/// Extract a cross-word through (row, col) in the given direction.
fn extract_cross_word(
    board: &Board,
    tiles: &[PlacedTile],
    row: usize,
    col: usize,
    direction: Direction,
) -> Vec<(usize, usize)> {
    let (fixed, pos) = match direction {
        Direction::Horizontal => (row, col),
        Direction::Vertical => (col, row),
    };

    let mut start = pos;
    while start > 0 {
        let (r, c) = match direction {
            Direction::Horizontal => (fixed, start - 1),
            Direction::Vertical => (start - 1, fixed),
        };
        if tile_at(board, tiles, r, c).is_some() {
            start -= 1;
        } else {
            break;
        }
    }

    let mut end = pos;
    while end + 1 < BOARD_SIZE {
        let (r, c) = match direction {
            Direction::Horizontal => (fixed, end + 1),
            Direction::Vertical => (end + 1, fixed),
        };
        if tile_at(board, tiles, r, c).is_some() {
            end += 1;
        } else {
            break;
        }
    }

    (start..=end)
        .map(|p| match direction {
            Direction::Horizontal => (fixed, p),
            Direction::Vertical => (p, fixed),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn place(row: usize, col: usize, ch: char) -> PlacedTile {
        PlacedTile {
            row,
            col,
            tile: Tile::Letter(ch, crate::tile::letter_points(ch)),
        }
    }

    #[test]
    fn first_move_must_cover_center() {
        let board = Board::new();
        let tiles = vec![place(0, 0, 'A'), place(0, 1, 'B')];
        let result = validate_placement(&board, &tiles, true);
        assert!(result.is_err());
    }

    #[test]
    fn valid_first_move() {
        let board = Board::new();
        let tiles = vec![place(7, 6, 'H'), place(7, 7, 'I')];
        let result = validate_placement(&board, &tiles, true);
        assert!(result.is_ok());
        let vp = result.unwrap();
        assert_eq!(vp.word_strings, vec!["HI"]);
    }

    #[test]
    fn tiles_must_be_in_line() {
        let board = Board::new();
        let tiles = vec![place(7, 7, 'A'), place(8, 8, 'B')];
        let result = validate_placement(&board, &tiles, true);
        assert!(result.is_err());
    }

    #[test]
    fn second_move_must_be_adjacent() {
        let mut board = Board::new();
        board.set(7, 7, Tile::Letter('A', 1));
        let tiles = vec![place(0, 0, 'B'), place(0, 1, 'C')];
        let result = validate_placement(&board, &tiles, false);
        assert!(result.is_err());
    }

    #[test]
    fn cross_words_extracted() {
        let mut board = Board::new();
        // Place "HI" horizontally at row 7
        board.set(7, 7, Tile::Letter('H', 4));
        board.set(7, 8, Tile::Letter('I', 1));

        // Place "AT" vertically crossing the H
        let tiles = vec![place(6, 7, 'A'), place(8, 7, 'T')];
        let result = validate_placement(&board, &tiles, false);
        assert!(result.is_ok());
        let vp = result.unwrap();
        // Should form "AHT" vertically (primary) and no cross words for single tiles
        // Actually: primary word is AHT (vertical), no cross words since A and T don't extend horizontally
        assert!(vp.word_strings.contains(&"AHT".to_string()));
    }
}
