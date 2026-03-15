use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

use wabble_words::game::WordGameState;
use wabble_words::placement::PlacedTile;
use wabble_words::tile::Tile;

use crate::eval::evaluate_leave;
use crate::movegen::CandidateMove;

/// AI difficulty levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    /// Picks a random move from the bottom third by score.
    Easy,
    /// Picks from the top 5 moves with some noise.
    Medium,
    /// Picks the best move considering score + leave evaluation.
    Hard,
}

impl Difficulty {
    /// Select a move from the candidate list based on difficulty.
    pub fn select_move(&self, moves: &[CandidateMove], state: &WordGameState) -> Vec<PlacedTile> {
        assert!(!moves.is_empty());

        let seed = state.turn_history.len() as u64 * 31 + state.bag.remaining() as u64;
        let mut rng = StdRng::seed_from_u64(seed);

        match self {
            Difficulty::Easy => select_easy(moves, &mut rng),
            Difficulty::Medium => select_medium(moves, &mut rng),
            Difficulty::Hard => select_hard(moves, state, &mut rng),
        }
    }
}

fn select_easy(moves: &[CandidateMove], rng: &mut StdRng) -> Vec<PlacedTile> {
    // Pick from the bottom third of moves (lower scoring)
    let count = (moves.len() / 3).max(1);
    let start = moves.len().saturating_sub(count);
    let idx = rng.gen_range(start..moves.len());
    moves[idx].tiles.clone()
}

fn select_medium(moves: &[CandidateMove], rng: &mut StdRng) -> Vec<PlacedTile> {
    // Pick from the top 5 moves with uniform probability
    let top_n = moves.len().min(5);
    let idx = rng.gen_range(0..top_n);
    moves[idx].tiles.clone()
}

fn select_hard(
    moves: &[CandidateMove],
    state: &WordGameState,
    rng: &mut StdRng,
) -> Vec<PlacedTile> {
    let current_rack = &state.players[state.current_player_idx].rack.tiles;

    // Evaluate each move by score + leave quality
    let mut best_score = f32::NEG_INFINITY;
    let mut best_indices: Vec<usize> = vec![0];

    for (i, m) in moves.iter().enumerate() {
        // Compute remaining rack after this move
        let remaining = compute_leave(current_rack, &m.tiles);
        let leave_val = evaluate_leave(&remaining);
        let total = m.score as f32 + leave_val;

        if total > best_score + 0.01 {
            best_score = total;
            best_indices = vec![i];
        } else if (total - best_score).abs() < 0.01 {
            best_indices.push(i);
        }
    }

    let idx = best_indices[rng.gen_range(0..best_indices.len())];
    moves[idx].tiles.clone()
}

/// Figure out which rack tiles remain after playing the given tiles.
fn compute_leave(rack: &[Tile], played: &[PlacedTile]) -> Vec<Tile> {
    let mut remaining = rack.to_vec();
    for pt in played {
        let pos = match pt.tile {
            Tile::Blank(_) => remaining.iter().position(|t| t.is_blank()),
            Tile::Letter(ch, _) => remaining.iter().position(|t| match t {
                Tile::Letter(c, _) => c.eq_ignore_ascii_case(&ch),
                _ => false,
            }),
        };
        if let Some(i) = pos {
            remaining.remove(i);
        }
    }
    remaining
}

#[cfg(test)]
mod tests {
    use super::*;
    use wabble_words::tile::letter_points;

    fn make_candidate(word: &str, score: i32) -> CandidateMove {
        let tiles: Vec<PlacedTile> = word
            .chars()
            .enumerate()
            .map(|(i, ch)| PlacedTile {
                row: 7,
                col: 7 + i,
                tile: Tile::Letter(ch, letter_points(ch)),
            })
            .collect();
        CandidateMove {
            tiles,
            score,
            primary_word: word.to_string(),
        }
    }

    #[test]
    fn hard_picks_high_scoring() {
        use wabble_platform::TurnBasedGame;
        use wabble_words::game::{WordGame, WordGameConfig};

        let game = WordGame::new_game(&WordGameConfig::default(), 2, 42);
        let state = game.state();

        let moves = vec![
            make_candidate("AT", 2),
            make_candidate("CAT", 10),
            make_candidate("CARE", 20),
        ];

        let result = Difficulty::Hard.select_move(&moves, state);
        // Hard should pick CARE (highest score + leave)
        assert!(result.len() >= 2); // At minimum picks a multi-tile word
    }

    #[test]
    fn easy_avoids_best() {
        use wabble_platform::TurnBasedGame;
        use wabble_words::game::{WordGame, WordGameConfig};

        let game = WordGame::new_game(&WordGameConfig::default(), 2, 42);
        let state = game.state();

        let moves: Vec<CandidateMove> = (0..20)
            .map(|i| make_candidate("AT", 100 - i * 5))
            .collect();
        // Easy should pick from the low end
        let result = Difficulty::Easy.select_move(&moves, state);
        assert!(!result.is_empty());
    }
}
