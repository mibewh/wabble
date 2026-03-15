use bevy::prelude::*;
use wabble_platform::{GameAi, TurnBasedGame};
use wabble_words::game::WordAction;

use crate::app_states::AppState;
use crate::resources::{ActiveMatch, AiMoveTimer, AiOpponent, StatusMessage, TurnTransition};

/// System that triggers AI moves when it's the AI's turn.
#[allow(clippy::too_many_arguments)]
pub fn ai_turn_system(
    mut commands: Commands,
    time: Res<Time>,
    active_match: Option<ResMut<ActiveMatch>>,
    ai_opponent: Option<Res<AiOpponent>>,
    mut ai_timer: Option<ResMut<AiMoveTimer>>,
    transition: Option<Res<TurnTransition>>,
    mut status: Option<ResMut<StatusMessage>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Some(mut active_match) = active_match else {
        return;
    };
    let Some(ai_opponent) = ai_opponent else {
        return;
    };

    // Don't act during turn transition
    if let Some(ref t) = transition
        && t.active {
            return;
        }

    let game = &active_match.game;
    if game.is_finished() {
        return;
    }

    let current = game.state().current_player_idx;
    if current != ai_opponent.player_idx {
        return;
    }

    // Start or tick the delay timer
    match ai_timer {
        Some(ref mut timer) => {
            timer.timer.tick(time.delta());
            if !timer.timer.is_finished() {
                return;
            }
            commands.remove_resource::<AiMoveTimer>();
        }
        None => {
            // Start a 0.8 second delay before AI moves
            commands.insert_resource(AiMoveTimer {
                timer: Timer::from_seconds(0.8, TimerMode::Once),
            });
            if let Some(ref mut status) = status {
                status.text = "AI is thinking...".to_string();
            }
            return;
        }
    }

    // AI makes its move
    let ai = &ai_opponent.ai;
    let action = ai
        .choose_action(&active_match.game, current)
        .unwrap_or(WordAction::Pass);

    match active_match.game.apply_action(current, action) {
        Ok(result) => {
            if let Some(ref mut status) = status {
                status.text = format!("AI {}", result.turn_summary);
            }

            if active_match.game.is_finished() {
                next_state.set(AppState::GameOver);
            }
        }
        Err(e) => {
            // If the AI's move was rejected, fall back to passing
            warn!("AI move rejected ({e}), falling back to pass");
            let _ = active_match.game.apply_action(current, WordAction::Pass);
            if let Some(ref mut status) = status {
                status.text = "AI passed".to_string();
            }
            if active_match.game.is_finished() {
                next_state.set(AppState::GameOver);
            }
        }
    }
}
