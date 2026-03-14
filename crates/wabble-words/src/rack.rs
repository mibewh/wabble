use serde::{Deserialize, Serialize};

use crate::tile::Tile;

pub const RACK_SIZE: usize = 7;

/// A player's tile rack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rack {
    pub tiles: Vec<Tile>,
}

impl Rack {
    pub fn new() -> Self {
        Self {
            tiles: Vec::with_capacity(RACK_SIZE),
        }
    }

    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }

    /// Remove a specific tile from the rack. For blanks, matches any blank.
    /// For letters, matches by character (case-insensitive).
    /// Returns the removed tile, or None if not found.
    pub fn remove_tile(&mut self, tile: &Tile) -> Option<Tile> {
        let pos = match tile {
            Tile::Blank(_) => self.tiles.iter().position(|t| t.is_blank()),
            Tile::Letter(ch, _) => self.tiles.iter().position(|t| match t {
                Tile::Letter(c, _) => c.eq_ignore_ascii_case(ch),
                _ => false,
            }),
        };
        pos.map(|i| self.tiles.remove(i))
    }

    /// Add tiles to the rack.
    pub fn add_tiles(&mut self, tiles: Vec<Tile>) {
        self.tiles.extend(tiles);
    }

    /// How many tiles needed to fill the rack to 7.
    pub fn tiles_needed(&self) -> usize {
        RACK_SIZE.saturating_sub(self.tiles.len())
    }
}

impl Default for Rack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rack_is_empty() {
        let rack = Rack::new();
        assert!(rack.is_empty());
        assert_eq!(rack.tiles_needed(), 7);
    }

    #[test]
    fn remove_letter_tile() {
        let mut rack = Rack::new();
        rack.add_tiles(vec![
            Tile::Letter('A', 1),
            Tile::Letter('B', 3),
            Tile::Letter('C', 3),
        ]);
        let removed = rack.remove_tile(&Tile::Letter('B', 3));
        assert!(removed.is_some());
        assert_eq!(rack.len(), 2);
    }

    #[test]
    fn remove_blank_tile() {
        let mut rack = Rack::new();
        rack.add_tiles(vec![Tile::Letter('A', 1), Tile::Blank(None)]);
        let removed = rack.remove_tile(&Tile::Blank(Some('X')));
        assert!(removed.is_some());
        assert!(removed.unwrap().is_blank());
        assert_eq!(rack.len(), 1);
    }
}
