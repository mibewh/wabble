use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

/// A tile: either a letter with its inherent point value, or a blank (wildcard).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Letter(char, u8),
    Blank(Option<char>),
}

impl Tile {
    /// The display character of this tile.
    pub fn letter(&self) -> Option<char> {
        match self {
            Tile::Letter(ch, _) => Some(*ch),
            Tile::Blank(ch) => *ch,
        }
    }

    /// Point value of this tile (blanks are always 0).
    pub fn points(&self) -> u8 {
        match self {
            Tile::Letter(_, pts) => *pts,
            Tile::Blank(_) => 0,
        }
    }

    /// Whether this tile is a blank.
    pub fn is_blank(&self) -> bool {
        matches!(self, Tile::Blank(_))
    }
}

/// Point values for each letter.
pub fn letter_points(ch: char) -> u8 {
    match ch.to_ascii_uppercase() {
        'A' | 'E' | 'I' | 'O' | 'U' | 'L' | 'N' | 'S' | 'T' | 'R' => 1,
        'D' | 'G' => 2,
        'B' | 'C' | 'M' | 'P' => 3,
        'F' | 'H' | 'V' | 'W' | 'Y' => 4,
        'K' => 5,
        'J' | 'X' => 8,
        'Q' | 'Z' => 10,
        _ => 0,
    }
}

/// Standard 100-tile bag distribution.
fn standard_distribution() -> Vec<Tile> {
    let letters: &[(char, u8)] = &[
        ('A', 9), ('B', 2), ('C', 2), ('D', 4), ('E', 12), ('F', 2),
        ('G', 3), ('H', 2), ('I', 9), ('J', 1), ('K', 1), ('L', 4),
        ('M', 2), ('N', 6), ('O', 8), ('P', 2), ('Q', 1), ('R', 6),
        ('S', 4), ('T', 6), ('U', 4), ('V', 2), ('W', 2), ('X', 1),
        ('Y', 2), ('Z', 1),
    ];
    let mut tiles = Vec::with_capacity(100);
    for &(ch, count) in letters {
        for _ in 0..count {
            tiles.push(Tile::Letter(ch, letter_points(ch)));
        }
    }
    // 2 blanks
    tiles.push(Tile::Blank(None));
    tiles.push(Tile::Blank(None));
    tiles
}

/// The tile bag from which players draw.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileBag {
    tiles: Vec<Tile>,
}

impl TileBag {
    /// Create a new standard bag, shuffled with the given seed.
    pub fn new(seed: u64) -> Self {
        let mut tiles = standard_distribution();
        let mut rng = StdRng::seed_from_u64(seed);
        tiles.shuffle(&mut rng);
        Self { tiles }
    }

    /// Draw up to `n` tiles from the bag.
    pub fn draw(&mut self, n: usize) -> Vec<Tile> {
        let count = n.min(self.tiles.len());
        self.tiles.split_off(self.tiles.len() - count)
    }

    /// Return tiles to the bag and reshuffle.
    pub fn return_tiles(&mut self, tiles: Vec<Tile>, seed: u64) {
        self.tiles.extend(tiles);
        let mut rng = StdRng::seed_from_u64(seed);
        self.tiles.shuffle(&mut rng);
    }

    /// Number of tiles remaining.
    pub fn remaining(&self) -> usize {
        self.tiles.len()
    }

    /// Whether the bag is empty.
    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_bag_has_100_tiles() {
        let bag = TileBag::new(42);
        assert_eq!(bag.remaining(), 100);
    }

    #[test]
    fn standard_bag_has_2_blanks() {
        let tiles = standard_distribution();
        let blanks = tiles.iter().filter(|t| t.is_blank()).count();
        assert_eq!(blanks, 2);
    }

    #[test]
    fn draw_reduces_count() {
        let mut bag = TileBag::new(42);
        let drawn = bag.draw(7);
        assert_eq!(drawn.len(), 7);
        assert_eq!(bag.remaining(), 93);
    }

    #[test]
    fn draw_from_empty_bag() {
        let mut bag = TileBag::new(42);
        let _ = bag.draw(100);
        let drawn = bag.draw(5);
        assert!(drawn.is_empty());
    }

    #[test]
    fn letter_points_correct() {
        assert_eq!(letter_points('A'), 1);
        assert_eq!(letter_points('Z'), 10);
        assert_eq!(letter_points('K'), 5);
        assert_eq!(letter_points('Q'), 10);
    }
}
