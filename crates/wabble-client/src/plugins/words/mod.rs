pub mod board;
pub mod hud;
pub mod input;
pub mod local_ai;
pub mod rack;

use bevy::prelude::*;

use crate::app_states::AppState;

pub struct WordsGamePlugin;

impl Plugin for WordsGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::InGame),
            (board::spawn_board, rack::spawn_rack, hud::spawn_hud),
        )
        .add_systems(
            Update,
            (
                input::handle_board_click,
                input::handle_rack_click,
                input::handle_play_button,
                input::handle_pass_button,
                input::handle_recall_button,
                input::handle_turn_transition,
                board::update_board_display,
                rack::update_rack_display,
                hud::update_score_display,
                hud::update_status_display,
                hud::update_button_colors,
                local_ai::ai_turn_system,
            )
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            OnExit(AppState::InGame),
            (board::cleanup_board, rack::cleanup_rack, hud::cleanup_hud),
        );
    }
}
