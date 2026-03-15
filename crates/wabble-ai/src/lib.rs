pub mod difficulty;
pub mod eval;
pub mod movegen;

use wabble_dict::{FstDictionary, Gaddag};
use wabble_platform::{GameAi, TurnBasedGame};
use wabble_words::game::WordAction;
use wabble_words::WordGame;

use difficulty::Difficulty;
use movegen::generate_moves;

/// AI opponent for the word game.
pub struct WordGameAi {
    pub gaddag: Gaddag,
    pub dictionary: FstDictionary,
    pub difficulty: Difficulty,
}

impl WordGameAi {
    pub fn new(gaddag: Gaddag, dictionary: FstDictionary, difficulty: Difficulty) -> Self {
        Self {
            gaddag,
            dictionary,
            difficulty,
        }
    }
}

impl GameAi<WordGame> for WordGameAi {
    fn choose_action(&self, game: &WordGame, player_idx: usize) -> Option<WordAction> {
        let state = game.state();
        if state.finished {
            return None;
        }
        if state.current_player_idx != player_idx {
            return None;
        }

        let player = game.player_state(player_idx)?;
        let rack = &player.rack;

        let moves = generate_moves(&state.board, &rack.tiles, &self.gaddag, &self.dictionary);

        if moves.is_empty() {
            return Some(WordAction::Pass);
        }

        let chosen = self.difficulty.select_move(&moves, state);
        Some(WordAction::Place(chosen))
    }
}
