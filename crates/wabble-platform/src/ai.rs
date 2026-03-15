use crate::game_trait::TurnBasedGame;

/// Trait for AI opponents that can play any turn-based game.
pub trait GameAi<G: TurnBasedGame>: Send + Sync {
    /// Choose an action for the given player. Returns None if the AI
    /// cannot determine a valid move (e.g., game is over).
    fn choose_action(&self, game: &G, player_idx: usize) -> Option<G::Action>;
}
