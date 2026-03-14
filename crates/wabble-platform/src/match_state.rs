use serde::{Deserialize, Serialize};

use crate::game_trait::TurnBasedGame;
use crate::player::PlayerId;

/// Result of applying a single turn action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnResult {
    pub score_delta: i32,
    pub turn_summary: String,
}

/// Final results of a completed game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameResults {
    pub player_scores: Vec<i32>,
    /// Index of the winner. None = draw.
    pub winner: Option<usize>,
}

/// Phase of a match lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchPhase {
    WaitingForPlayers,
    InProgress,
    Finished,
}

/// Game-agnostic match wrapper.
#[derive(Debug, Clone)]
pub struct Match<G: TurnBasedGame> {
    pub game: G,
    pub phase: MatchPhase,
    pub player_ids: Vec<PlayerId>,
    pub turn_number: u32,
    pub created_at: u64,
    pub updated_at: u64,
}

impl<G: TurnBasedGame> Match<G> {
    pub fn new(game: G, player_ids: Vec<PlayerId>, now: u64) -> Self {
        let phase = if player_ids.len() >= G::player_count_range().0 {
            MatchPhase::InProgress
        } else {
            MatchPhase::WaitingForPlayers
        };
        Self {
            game,
            phase,
            player_ids,
            turn_number: 0,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn apply_action(
        &mut self,
        player_idx: usize,
        action: G::Action,
        now: u64,
    ) -> Result<TurnResult, G::Error> {
        let result = self.game.apply_action(player_idx, action)?;
        self.turn_number += 1;
        self.updated_at = now;
        if self.game.is_finished() {
            self.phase = MatchPhase::Finished;
        }
        Ok(result)
    }
}
