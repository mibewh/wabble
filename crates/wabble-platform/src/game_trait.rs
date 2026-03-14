use serde::{Serialize, de::DeserializeOwned};

use crate::match_state::{GameResults, TurnResult};

/// A turn-based game that can be plugged into the platform.
pub trait TurnBasedGame: Clone + Send + Sync {
    /// Opaque game state (board, pieces, etc.)
    type State: Clone + Send + Sync + Serialize + DeserializeOwned;
    /// A player's private state (hand, hidden info)
    type PlayerState: Clone + Send + Sync + Serialize + DeserializeOwned;
    /// An action a player can take on their turn
    type Action: Clone + Send + Sync + Serialize + DeserializeOwned;
    /// Public view of game state (what spectators/opponents can see)
    type PublicView: Clone + Send + Sync + Serialize + DeserializeOwned;
    /// Game configuration / settings
    type Config: Clone + Send + Sync + Serialize + DeserializeOwned + Default;
    /// Game-specific error type
    type Error: std::error::Error + Send + Sync;

    /// Create a new game with the given number of players.
    fn new_game(config: &Self::Config, num_players: usize, rng_seed: u64) -> Self;

    /// Get the full game state.
    fn state(&self) -> &Self::State;

    /// Get a player's private state (rack, hand, etc.).
    fn player_state(&self, player_idx: usize) -> Option<&Self::PlayerState>;

    /// Get the public view (for spectators/opponents).
    fn public_view(&self) -> Self::PublicView;

    /// Which player's turn is it? None if game is over.
    fn current_player(&self) -> Option<usize>;

    /// Validate and apply an action. Returns score delta (if applicable).
    fn apply_action(
        &mut self,
        player_idx: usize,
        action: Self::Action,
    ) -> Result<TurnResult, Self::Error>;

    /// Is the game finished?
    fn is_finished(&self) -> bool;

    /// Final results (scores, winner, etc.).
    fn results(&self) -> Option<GameResults>;

    /// Number of players.
    fn num_players(&self) -> usize;

    /// Minimum/maximum player count for this game type.
    fn player_count_range() -> (usize, usize);
}
