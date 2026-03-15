use bevy::prelude::*;
use wabble_words::game::WordGame;
use wabble_words::placement::PlacedTile;
use wabble_words::tile::Tile;

/// The active game match.
#[derive(Resource)]
pub struct ActiveMatch {
    pub game: WordGame,
    pub player_count: usize,
}

/// Tiles placed on the board this turn but not yet submitted.
#[derive(Resource, Default)]
pub struct PendingPlacement {
    pub tiles: Vec<PendingTile>,
}

/// A tile placed from rack to board, not yet committed.
pub struct PendingTile {
    pub row: usize,
    pub col: usize,
    pub tile: Tile,
    pub rack_index: usize,
}

impl PendingPlacement {
    pub fn to_placed_tiles(&self) -> Vec<PlacedTile> {
        self.tiles
            .iter()
            .map(|pt| PlacedTile {
                row: pt.row,
                col: pt.col,
                tile: pt.tile,
            })
            .collect()
    }

    pub fn is_at(&self, row: usize, col: usize) -> bool {
        self.tiles.iter().any(|pt| pt.row == row && pt.col == col)
    }

    pub fn remove_at(&mut self, row: usize, col: usize) -> Option<PendingTile> {
        let pos = self
            .tiles
            .iter()
            .position(|pt| pt.row == row && pt.col == col)?;
        Some(self.tiles.remove(pos))
    }
}

/// Which rack tile is currently selected for placement.
#[derive(Resource, Default)]
pub struct SelectedRackTile {
    pub index: Option<usize>,
}

/// Turn transition state - shown between hot-seat turns.
#[derive(Resource)]
#[derive(Default)]
pub struct TurnTransition {
    pub next_player: usize,
    pub active: bool,
}


/// Optional dictionary for word validation.
#[derive(Resource)]
pub struct GameDictionary {
    pub dict: wabble_dict::FstDictionary,
}

/// Last action result message to display.
#[derive(Resource, Default)]
pub struct StatusMessage {
    pub text: String,
}

/// Marks that this game has an AI opponent and which player index it controls.
#[derive(Resource)]
pub struct AiOpponent {
    pub player_idx: usize,
    pub ai: std::sync::Arc<wabble_ai::WordGameAi>,
}

/// Timer to add a small delay before AI moves, so the player can see what happened.
#[derive(Resource)]
pub struct AiMoveTimer {
    pub timer: bevy::time::Timer,
}
