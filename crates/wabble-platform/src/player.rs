use serde::{Deserialize, Serialize};

/// Unique player identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub u64);

/// Player profile with stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProfile {
    pub id: PlayerId,
    pub display_name: String,
    pub elo: i32,
    pub games_played: u32,
    pub games_won: u32,
}

impl PlayerProfile {
    pub fn new(id: PlayerId, display_name: String) -> Self {
        Self {
            id,
            display_name,
            elo: 1200,
            games_played: 0,
            games_won: 0,
        }
    }
}
