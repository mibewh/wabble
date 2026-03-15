use bevy::prelude::*;
use wabble_platform::TurnBasedGame;
use wabble_words::game::WordAction;

use crate::app_states::AppState;
use crate::resources::{
    ActiveMatch, PendingPlacement, PendingTile, SelectedRackTile, StatusMessage, TurnTransition,
};

use super::board::world_to_grid;
use super::rack::world_to_rack_index;

#[derive(Component)]
pub struct PlayButton;

#[derive(Component)]
pub struct PassButton;

#[derive(Component)]
pub struct RecallButton;

#[derive(Component)]
pub struct TransitionOverlay;

#[derive(Component)]
pub struct TransitionReadyButton;

pub fn handle_board_click(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    active_match: Option<Res<ActiveMatch>>,
    mut pending: Option<ResMut<PendingPlacement>>,
    mut selected: Option<ResMut<SelectedRackTile>>,
    transition: Option<Res<TurnTransition>>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    // Don't process during turn transition
    if let Some(ref t) = transition
        && t.active {
            return;
        }

    let Some(active_match) = active_match else {
        return;
    };
    let Some(ref mut pending) = pending else {
        return;
    };
    let Some(ref mut selected) = selected else {
        return;
    };

    let window = windows.single();
    let (camera, camera_transform) = cameras.single();

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    let Some((row, col)) = world_to_grid(world_pos) else {
        return;
    };

    // If clicking on a pending tile, recall it
    if pending.is_at(row, col) {
        pending.remove_at(row, col);
        selected.index = None;
        return;
    }

    // If a rack tile is selected and the cell is empty, place it
    if let Some(rack_idx) = selected.index
        && active_match.game.state().board.is_empty_at(row, col) && !pending.is_at(row, col) {
            let current_player = active_match.game.state().current_player_idx;
            if let Some(ps) = active_match.game.player_state(current_player)
                && rack_idx < ps.rack.len() {
                    let tile = ps.rack.tiles[rack_idx];
                    pending.tiles.push(PendingTile {
                        row,
                        col,
                        tile,
                        rack_index: rack_idx,
                    });
                    selected.index = None;
                }
        }
}

pub fn handle_rack_click(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    active_match: Option<Res<ActiveMatch>>,
    mut selected: Option<ResMut<SelectedRackTile>>,
    pending: Option<Res<PendingPlacement>>,
    transition: Option<Res<TurnTransition>>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    if let Some(ref t) = transition
        && t.active {
            return;
        }

    let Some(active_match) = active_match else {
        return;
    };
    let Some(ref mut selected) = selected else {
        return;
    };

    let window = windows.single();
    let (camera, camera_transform) = cameras.single();

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    let current_player = active_match.game.state().current_player_idx;
    let Some(ps) = active_match.game.player_state(current_player) else {
        return;
    };

    // Check which rack tiles are not pending
    let pending_indices: Vec<usize> = pending
        .as_ref()
        .map(|p| p.tiles.iter().map(|t| t.rack_index).collect())
        .unwrap_or_default();

    if let Some(idx) = world_to_rack_index(world_pos, ps.rack.len())
        && !pending_indices.contains(&idx) {
            if selected.index == Some(idx) {
                selected.index = None; // Deselect
            } else {
                selected.index = Some(idx);
            }
        }
}

pub fn handle_play_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<PlayButton>)>,
    mut active_match: Option<ResMut<ActiveMatch>>,
    mut pending: Option<ResMut<PendingPlacement>>,
    mut selected: Option<ResMut<SelectedRackTile>>,
    mut status: Option<ResMut<StatusMessage>>,
    mut transition: Option<ResMut<TurnTransition>>,
) {
    for interaction in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Some(ref mut active_match) = active_match else {
            continue;
        };
        let Some(ref mut pending) = pending else {
            continue;
        };

        if pending.tiles.is_empty() {
            if let Some(ref mut status) = status {
                status.text = "Place some tiles first!".to_string();
            }
            continue;
        }

        let placed = pending.to_placed_tiles();
        let player = active_match.game.state().current_player_idx;

        match active_match
            .game
            .apply_action(player, WordAction::Place(placed))
        {
            Ok(result) => {
                if let Some(ref mut status) = status {
                    status.text = result.turn_summary.clone();
                }
                pending.tiles.clear();
                if let Some(ref mut selected) = selected {
                    selected.index = None;
                }

                // Start turn transition
                if !active_match.game.is_finished()
                    && let Some(ref mut t) = transition {
                        t.next_player = active_match.game.state().current_player_idx;
                        t.active = true;
                    }
            }
            Err(e) => {
                if let Some(ref mut status) = status {
                    status.text = format!("Invalid: {e}");
                }
            }
        }
    }
}

pub fn handle_pass_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<PassButton>)>,
    mut active_match: Option<ResMut<ActiveMatch>>,
    mut pending: Option<ResMut<PendingPlacement>>,
    mut selected: Option<ResMut<SelectedRackTile>>,
    mut status: Option<ResMut<StatusMessage>>,
    mut transition: Option<ResMut<TurnTransition>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Some(ref mut active_match) = active_match else {
            continue;
        };

        let player = active_match.game.state().current_player_idx;
        match active_match.game.apply_action(player, WordAction::Pass) {
            Ok(result) => {
                if let Some(ref mut status) = status {
                    status.text = result.turn_summary.clone();
                }
                if let Some(ref mut pending) = pending {
                    pending.tiles.clear();
                }
                if let Some(ref mut selected) = selected {
                    selected.index = None;
                }

                if active_match.game.is_finished() {
                    next_state.set(AppState::GameOver);
                } else if let Some(ref mut t) = transition {
                    t.next_player = active_match.game.state().current_player_idx;
                    t.active = true;
                }
            }
            Err(e) => {
                if let Some(ref mut status) = status {
                    status.text = format!("Error: {e}");
                }
            }
        }
    }
}

pub fn handle_recall_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<RecallButton>)>,
    mut pending: Option<ResMut<PendingPlacement>>,
    mut selected: Option<ResMut<SelectedRackTile>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            if let Some(ref mut pending) = pending {
                pending.tiles.clear();
            }
            if let Some(ref mut selected) = selected {
                selected.index = None;
            }
        }
    }
}

pub fn handle_turn_transition(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<TransitionReadyButton>)>,
    mut transition: Option<ResMut<TurnTransition>>,
    overlay_query: Query<Entity, With<TransitionOverlay>>,
    active_match: Option<Res<ActiveMatch>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // Show overlay when transition becomes active
    if let Some(ref transition) = transition
        && transition.active && transition.is_changed() && overlay_query.is_empty() {
            let player_num = transition.next_player + 1;
            commands
                .spawn((
                    TransitionOverlay,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        row_gap: Val::Px(30.0),
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
                    GlobalZIndex(100),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(format!("Player {player_num}'s Turn")),
                        TextFont {
                            font_size: 48.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    parent
                        .spawn((
                            TransitionReadyButton,
                            Button,
                            Node {
                                width: Val::Px(200.0),
                                height: Val::Px(60.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.6, 0.3)),
                            BorderRadius::all(Val::Px(8.0)),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("Ready"),
                                TextFont {
                                    font_size: 28.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                });
        }

    // Handle "Ready" button press
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            if let Some(ref mut transition) = transition {
                transition.active = false;
            }
            for entity in &overlay_query {
                commands.entity(entity).despawn_recursive();
            }

            // Check if game is over
            if let Some(ref am) = active_match
                && am.game.is_finished() {
                    next_state.set(AppState::GameOver);
                }
        }
    }
}
