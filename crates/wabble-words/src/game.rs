use serde::{Deserialize, Serialize};
use wabble_platform::{GameResults, TurnBasedGame, TurnResult};

use crate::board::Board;
use crate::error::WordGameError;
use crate::placement::{PlacedTile, validate_placement};
use crate::rack::Rack;
use crate::rules::{MAX_CONSECUTIVE_ZERO_TURNS, MIN_BAG_FOR_EXCHANGE};
use crate::scoring;
use crate::tile::{Tile, TileBag};

/// Trait for word validation — allows mock implementations in tests.
pub trait WordValidator: Send + Sync {
    fn is_valid_word(&self, word: &str) -> bool;
}

/// A validator that accepts all words (for testing / local play without dictionary).
#[derive(Debug, Clone)]
pub struct AcceptAllValidator;

impl WordValidator for AcceptAllValidator {
    fn is_valid_word(&self, _word: &str) -> bool {
        true
    }
}

/// Game configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WordGameConfig {
    /// Whether to validate words (false = accept all).
    pub validate_words: bool,
}

/// An action a player can take.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WordAction {
    /// Place tiles on the board.
    Place(Vec<PlacedTile>),
    /// Exchange tiles (return these, draw new ones).
    Exchange(Vec<Tile>),
    /// Pass the turn.
    Pass,
}

/// Full game state (serializable).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordGameState {
    pub board: Board,
    pub bag: TileBag,
    pub players: Vec<WordPlayerState>,
    pub current_player_idx: usize,
    pub consecutive_zero_turns: u8,
    pub finished: bool,
    pub turn_history: Vec<TurnRecord>,
}

/// Per-player state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordPlayerState {
    pub rack: Rack,
    pub score: i32,
}

/// Record of a turn for history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnRecord {
    pub player_idx: usize,
    pub action_type: String,
    pub score_delta: i32,
    pub words_formed: Vec<String>,
}

/// Public view (no racks visible).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordPublicView {
    pub board: Board,
    pub scores: Vec<i32>,
    pub current_player_idx: usize,
    pub tiles_remaining: usize,
    pub finished: bool,
    pub turn_history: Vec<TurnRecord>,
}

/// The word game itself.
#[derive(Debug, Clone)]
pub struct WordGame {
    state: WordGameState,
    config: WordGameConfig,
    /// RNG seed, incremented for each random operation.
    rng_counter: u64,
}

impl WordGame {
    /// Create a new game with a custom validator setup.
    fn init(config: &WordGameConfig, num_players: usize, rng_seed: u64) -> Self {
        assert!(
            (2..=4).contains(&num_players),
            "word game supports 2-4 players"
        );

        let mut bag = TileBag::new(rng_seed);
        let mut players = Vec::with_capacity(num_players);

        for _ in 0..num_players {
            let mut rack = Rack::new();
            let drawn = bag.draw(7);
            rack.add_tiles(drawn);
            players.push(WordPlayerState { rack, score: 0 });
        }

        Self {
            state: WordGameState {
                board: Board::new(),
                bag,
                players,
                current_player_idx: 0,
                consecutive_zero_turns: 0,
                finished: false,
                turn_history: Vec::new(),
            },
            config: config.clone(),
            rng_counter: rng_seed.wrapping_add(1),
        }
    }

    fn next_rng(&mut self) -> u64 {
        let seed = self.rng_counter;
        self.rng_counter = self.rng_counter.wrapping_add(1);
        seed
    }

    fn advance_turn(&mut self) {
        self.state.current_player_idx =
            (self.state.current_player_idx + 1) % self.state.players.len();
    }

    fn check_game_end(&mut self) {
        // Game ends if: consecutive zero turns >= 6
        if self.state.consecutive_zero_turns >= MAX_CONSECUTIVE_ZERO_TURNS {
            self.finish_game();
            return;
        }

        // Game ends if a player has emptied their rack and bag is empty
        if self.state.bag.is_empty()
            && self.state.players.iter().any(|p| p.rack.is_empty())
        {
            self.finish_game();
        }
    }

    fn finish_game(&mut self) {
        self.state.finished = true;

        // Subtract remaining rack values from each player
        // The player who went out gets the sum of all other racks added
        let rack_values: Vec<i32> = self
            .state
            .players
            .iter()
            .map(|p| scoring::rack_penalty(&p.rack.tiles))
            .collect();

        let empty_player = self
            .state
            .players
            .iter()
            .position(|p| p.rack.is_empty());

        for (i, player) in self.state.players.iter_mut().enumerate() {
            player.score -= rack_values[i];
        }

        if let Some(winner_idx) = empty_player {
            let bonus: i32 = rack_values.iter().sum();
            self.state.players[winner_idx].score += bonus;
        }
    }

    /// Apply a Place action. `validator` is used to check words if config requires it.
    fn apply_place(
        &mut self,
        player_idx: usize,
        tiles: Vec<PlacedTile>,
        validator: Option<&dyn WordValidator>,
    ) -> Result<TurnResult, WordGameError> {
        let is_first_move = self.state.board.is_empty();
        let validated = validate_placement(&self.state.board, &tiles, is_first_move)?;

        // Validate words if configured
        if self.config.validate_words
            && let Some(v) = validator
        {
            for word in &validated.word_strings {
                if !v.is_valid_word(word) {
                    return Err(WordGameError::InvalidWord(word.clone()));
                }
            }
        }

        // Remove tiles from rack
        let rack = &mut self.state.players[player_idx].rack;
        let mut tiles_to_place: Vec<(usize, usize, Tile)> = Vec::new();

        for pt in &tiles {
            let to_remove = &pt.tile;
            let removed = rack.remove_tile(to_remove);
            if removed.is_none() {
                return Err(WordGameError::TileNotInRack(
                    to_remove.letter().unwrap_or('?'),
                ));
            }
            tiles_to_place.push((pt.row, pt.col, pt.tile));
        }

        // Score all words
        let mut total_score = 0i32;
        for word_positions in &validated.words_formed {
            total_score += scoring::score_word(word_positions, &self.state.board, &tiles, &tiles_to_place);
        }
        total_score += scoring::bingo_bonus(tiles.len());

        // Place tiles on board
        for (row, col, tile) in &tiles_to_place {
            self.state.board.set(*row, *col, *tile);
        }

        // Draw new tiles
        let needed = rack.tiles_needed();
        let drawn = self.state.bag.draw(needed);
        self.state.players[player_idx].rack.add_tiles(drawn);

        // Update score
        self.state.players[player_idx].score += total_score;
        self.state.consecutive_zero_turns = 0;

        let summary = format!(
            "played {} for {} points",
            validated.word_strings.join(", "),
            total_score
        );

        self.state.turn_history.push(TurnRecord {
            player_idx,
            action_type: "place".into(),
            score_delta: total_score,
            words_formed: validated.word_strings,
        });

        self.advance_turn();
        self.check_game_end();

        Ok(TurnResult {
            score_delta: total_score,
            turn_summary: summary,
        })
    }

    fn apply_exchange(
        &mut self,
        player_idx: usize,
        tiles: Vec<Tile>,
    ) -> Result<TurnResult, WordGameError> {
        if self.state.bag.remaining() < MIN_BAG_FOR_EXCHANGE {
            return Err(WordGameError::ExchangeNotAllowed);
        }

        if tiles.is_empty() {
            return Err(WordGameError::InvalidPlacement(
                "must exchange at least one tile".into(),
            ));
        }

        // Remove tiles from rack
        let rack = &mut self.state.players[player_idx].rack;
        let mut removed = Vec::new();
        for tile in &tiles {
            match rack.remove_tile(tile) {
                Some(t) => removed.push(t),
                None => return Err(WordGameError::TileNotInRack(tile.letter().unwrap_or('?'))),
            }
        }

        // Draw new tiles first, then return old ones
        let drawn = self.state.bag.draw(removed.len());
        self.state.players[player_idx].rack.add_tiles(drawn);
        let seed = self.next_rng();
        self.state.bag.return_tiles(removed, seed);

        self.state.consecutive_zero_turns += 1;

        self.state.turn_history.push(TurnRecord {
            player_idx,
            action_type: "exchange".into(),
            score_delta: 0,
            words_formed: vec![],
        });

        let summary = format!("exchanged {} tiles", tiles.len());
        self.advance_turn();
        self.check_game_end();

        Ok(TurnResult {
            score_delta: 0,
            turn_summary: summary,
        })
    }

    fn apply_pass(&mut self, player_idx: usize) -> Result<TurnResult, WordGameError> {
        self.state.consecutive_zero_turns += 1;

        self.state.turn_history.push(TurnRecord {
            player_idx,
            action_type: "pass".into(),
            score_delta: 0,
            words_formed: vec![],
        });

        self.advance_turn();
        self.check_game_end();

        Ok(TurnResult {
            score_delta: 0,
            turn_summary: "passed".into(),
        })
    }

    /// Apply an action with an optional word validator.
    pub fn apply_action_with_validator(
        &mut self,
        player_idx: usize,
        action: WordAction,
        validator: Option<&dyn WordValidator>,
    ) -> Result<TurnResult, WordGameError> {
        if self.state.finished {
            return Err(WordGameError::GameAlreadyFinished);
        }
        if player_idx != self.state.current_player_idx {
            return Err(WordGameError::NotYourTurn);
        }
        if player_idx >= self.state.players.len() {
            return Err(WordGameError::InvalidPlayerIndex);
        }

        match action {
            WordAction::Place(tiles) => self.apply_place(player_idx, tiles, validator),
            WordAction::Exchange(tiles) => self.apply_exchange(player_idx, tiles),
            WordAction::Pass => self.apply_pass(player_idx),
        }
    }
}

impl TurnBasedGame for WordGame {
    type State = WordGameState;
    type PlayerState = WordPlayerState;
    type Action = WordAction;
    type PublicView = WordPublicView;
    type Config = WordGameConfig;
    type Error = WordGameError;

    fn new_game(config: &Self::Config, num_players: usize, rng_seed: u64) -> Self {
        Self::init(config, num_players, rng_seed)
    }

    fn state(&self) -> &Self::State {
        &self.state
    }

    fn player_state(&self, player_idx: usize) -> Option<&Self::PlayerState> {
        self.state.players.get(player_idx)
    }

    fn public_view(&self) -> Self::PublicView {
        WordPublicView {
            board: self.state.board.clone(),
            scores: self.state.players.iter().map(|p| p.score).collect(),
            current_player_idx: self.state.current_player_idx,
            tiles_remaining: self.state.bag.remaining(),
            finished: self.state.finished,
            turn_history: self.state.turn_history.clone(),
        }
    }

    fn current_player(&self) -> Option<usize> {
        if self.state.finished {
            None
        } else {
            Some(self.state.current_player_idx)
        }
    }

    fn apply_action(
        &mut self,
        player_idx: usize,
        action: Self::Action,
    ) -> Result<TurnResult, Self::Error> {
        // When called via the trait (no validator), use AcceptAll
        let validator = AcceptAllValidator;
        self.apply_action_with_validator(player_idx, action, Some(&validator))
    }

    fn is_finished(&self) -> bool {
        self.state.finished
    }

    fn results(&self) -> Option<GameResults> {
        if !self.state.finished {
            return None;
        }

        let scores: Vec<i32> = self.state.players.iter().map(|p| p.score).collect();
        let max_score = *scores.iter().max().unwrap_or(&0);
        let winners: Vec<usize> = scores
            .iter()
            .enumerate()
            .filter(|(_, s)| **s == max_score)
            .map(|(i, _)| i)
            .collect();

        let winner = if winners.len() == 1 {
            Some(winners[0])
        } else {
            None // draw
        };

        Some(GameResults {
            player_scores: scores,
            winner,
        })
    }

    fn num_players(&self) -> usize {
        self.state.players.len()
    }

    fn player_count_range() -> (usize, usize) {
        (2, 4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::placement::PlacedTile;

    fn new_game() -> WordGame {
        WordGame::new_game(&WordGameConfig::default(), 2, 42)
    }

    #[test]
    fn game_starts_correctly() {
        let game = new_game();
        assert_eq!(game.num_players(), 2);
        assert_eq!(game.current_player(), Some(0));
        assert!(!game.is_finished());
        // Each player drew 7, bag should have 86
        assert_eq!(game.state().bag.remaining(), 86);
        assert_eq!(game.player_state(0).unwrap().rack.len(), 7);
        assert_eq!(game.player_state(1).unwrap().rack.len(), 7);
    }

    #[test]
    fn wrong_player_cant_move() {
        let mut game = new_game();
        let result = game.apply_action(1, WordAction::Pass);
        assert!(matches!(result, Err(WordGameError::NotYourTurn)));
    }

    #[test]
    fn pass_advances_turn() {
        let mut game = new_game();
        let result = game.apply_action(0, WordAction::Pass);
        assert!(result.is_ok());
        assert_eq!(game.current_player(), Some(1));
    }

    #[test]
    fn six_consecutive_passes_ends_game() {
        let mut game = new_game();
        for i in 0..6 {
            let player = i % 2;
            game.apply_action(player, WordAction::Pass).unwrap();
        }
        assert!(game.is_finished());
        assert!(game.results().is_some());
    }

    #[test]
    fn place_first_word_scores() {
        let mut game = new_game();
        // We need to use tiles from player 0's rack
        let rack_tiles: Vec<Tile> = game.player_state(0).unwrap().rack.tiles.clone();

        // Place first two tiles on center
        if rack_tiles.len() >= 2 {
            let t0 = rack_tiles[0];
            let t1 = rack_tiles[1];
            let tiles = vec![
                PlacedTile { row: 7, col: 7, tile: t0 },
                PlacedTile { row: 7, col: 8, tile: t1 },
            ];
            let result = game.apply_action(0, WordAction::Place(tiles));
            assert!(result.is_ok());
            let tr = result.unwrap();
            assert!(tr.score_delta > 0 || tr.score_delta == 0); // blanks could score 0
            assert_eq!(game.current_player(), Some(1));
        }
    }

    #[test]
    fn exchange_tiles() {
        let mut game = new_game();
        let first_tile = game.player_state(0).unwrap().rack.tiles[0];
        let result = game.apply_action(0, WordAction::Exchange(vec![first_tile]));
        assert!(result.is_ok());
        assert_eq!(game.current_player(), Some(1));
        // Rack should still have 7 tiles
        assert_eq!(game.player_state(0).unwrap().rack.len(), 7);
    }

    #[test]
    fn public_view_hides_racks() {
        let game = new_game();
        let view = game.public_view();
        assert_eq!(view.scores.len(), 2);
        assert_eq!(view.tiles_remaining, 86);
        assert!(!view.finished);
    }

    #[test]
    fn tile_conservation() {
        let game = new_game();
        let bag_count = game.state().bag.remaining();
        let rack_count: usize = game
            .state()
            .players
            .iter()
            .map(|p| p.rack.len())
            .sum();
        // No tiles on board yet
        assert_eq!(bag_count + rack_count, 100);
    }

    #[test]
    fn full_game_simulation() {
        let mut game = new_game();
        let mut turn = 0;

        // Play a few real moves then pass to end
        while !game.is_finished() && turn < 200 {
            let player = game.current_player().unwrap();

            if turn < 2 {
                // First two turns: try to place words
                let rack = game.player_state(player).unwrap().rack.tiles.clone();
                if turn == 0 && rack.len() >= 2 {
                    let tiles = vec![
                        PlacedTile { row: 7, col: 7, tile: rack[0] },
                        PlacedTile { row: 7, col: 8, tile: rack[1] },
                    ];
                    let _ = game.apply_action(player, WordAction::Place(tiles));
                } else if turn == 1 && rack.len() >= 2 {
                    // Place crossing the first word
                    let tiles = vec![
                        PlacedTile { row: 6, col: 7, tile: rack[0] },
                        PlacedTile { row: 8, col: 7, tile: rack[1] },
                    ];
                    let result = game.apply_action(player, WordAction::Place(tiles));
                    if result.is_err() {
                        // Fallback to pass
                        game.apply_action(player, WordAction::Pass).unwrap();
                    }
                } else {
                    game.apply_action(player, WordAction::Pass).unwrap();
                }
            } else {
                // Just pass to end the game
                game.apply_action(player, WordAction::Pass).unwrap();
            }
            turn += 1;
        }

        assert!(game.is_finished());
        let results = game.results().unwrap();
        assert_eq!(results.player_scores.len(), 2);
    }

    #[test]
    fn word_validation_rejects_invalid() {
        struct RejectAll;
        impl WordValidator for RejectAll {
            fn is_valid_word(&self, _word: &str) -> bool {
                false
            }
        }

        let config = WordGameConfig {
            validate_words: true,
        };
        let mut game = WordGame::new_game(&config, 2, 42);
        let rack = game.player_state(0).unwrap().rack.tiles.clone();

        let tiles = vec![
            PlacedTile { row: 7, col: 7, tile: rack[0] },
            PlacedTile { row: 7, col: 8, tile: rack[1] },
        ];

        let result =
            game.apply_action_with_validator(0, WordAction::Place(tiles), Some(&RejectAll));
        assert!(matches!(result, Err(WordGameError::InvalidWord(_))));
    }

    #[test]
    fn word_validation_accepts_valid() {
        struct AcceptAll;
        impl WordValidator for AcceptAll {
            fn is_valid_word(&self, _word: &str) -> bool {
                true
            }
        }

        let config = WordGameConfig {
            validate_words: true,
        };
        let mut game = WordGame::new_game(&config, 2, 42);
        let rack = game.player_state(0).unwrap().rack.tiles.clone();

        let tiles = vec![
            PlacedTile { row: 7, col: 7, tile: rack[0] },
            PlacedTile { row: 7, col: 8, tile: rack[1] },
        ];

        let result =
            game.apply_action_with_validator(0, WordAction::Place(tiles), Some(&AcceptAll));
        assert!(result.is_ok());
    }
}
