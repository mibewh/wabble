use bevy::prelude::*;
use wabble_platform::TurnBasedGame;
use wabble_words::game::WordAction;

use crate::app_states::AppState;
use crate::resources::{
    ActiveMatch, DragInfo, DragState, PendingPlacement, PendingTile, SelectedRackTile,
    StatusMessage, TurnTransition,
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

#[derive(Component)]
pub struct DragGhost;

/// Get the current cursor world position.
fn cursor_world_pos(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = windows.single();
    let (camera, camera_transform) = cameras.single();
    let cursor_pos = window.cursor_position()?;
    camera
        .viewport_to_world_2d(camera_transform, cursor_pos)
        .ok()
}

/// Handle drag start (mouse press on rack tile), drag move, and drag end (mouse release).
#[allow(clippy::too_many_arguments)]
pub fn handle_drag(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    active_match: Option<Res<ActiveMatch>>,
    mut pending: Option<ResMut<PendingPlacement>>,
    mut drag_state: Option<ResMut<DragState>>,
    mut selected: Option<ResMut<SelectedRackTile>>,
    transition: Option<Res<TurnTransition>>,
) {
    // Don't process during turn transition
    if let Some(ref t) = transition
        && t.active
    {
        return;
    }

    let Some(active_match) = active_match else {
        return;
    };
    let Some(ref mut drag_state) = drag_state else {
        return;
    };

    let Some(world_pos) = cursor_world_pos(&windows, &cameras) else {
        return;
    };

    let current_player = active_match.game.state().current_player_idx;
    let Some(ps) = active_match.game.player_state(current_player) else {
        return;
    };

    let pending_indices: Vec<usize> = pending
        .as_ref()
        .map(|p| p.tiles.iter().map(|t| t.rack_index).collect())
        .unwrap_or_default();

    // --- Mouse just pressed: start drag or recall pending tile ---
    if mouse.just_pressed(MouseButton::Left) {
        // Check if clicking on a pending tile on the board to recall it
        if let Some(ref mut pending) = pending
            && let Some((row, col)) = world_to_grid(world_pos)
                && pending.is_at(row, col) {
                    pending.remove_at(row, col);
                    if let Some(ref mut selected) = selected {
                        selected.index = None;
                    }
                    return;
                }

        // Check if clicking on a rack tile to start dragging
        if let Some(idx) = world_to_rack_index(world_pos, ps.rack.len())
            && !pending_indices.contains(&idx) && idx < ps.rack.len() {
                let tile = ps.rack.tiles[idx];
                drag_state.dragging = Some(DragInfo {
                    rack_index: idx,
                    tile,
                    world_pos,
                });
                // Clear any click-based selection
                if let Some(ref mut selected) = selected {
                    selected.index = None;
                }
            }
    }

    // --- Mouse held: update drag position ---
    if mouse.pressed(MouseButton::Left)
        && let Some(ref mut info) = drag_state.dragging {
            info.world_pos = world_pos;
        }

    // --- Mouse released: drop tile or cancel ---
    if mouse.just_released(MouseButton::Left)
        && let Some(info) = drag_state.dragging.take() {
            // Check if released over a valid board cell
            if let Some((row, col)) = world_to_grid(world_pos)
                && let Some(ref mut pending) = pending
                    && active_match.game.state().board.is_empty_at(row, col)
                        && !pending.is_at(row, col)
                    {
                        pending.tiles.push(PendingTile {
                            row,
                            col,
                            tile: info.tile,
                            rack_index: info.rack_index,
                        });
                    }
            // If not over a valid cell, the drag is simply cancelled (tile returns to rack)
        }
}

/// Spawn/despawn the drag ghost sprite that follows the cursor.
pub fn update_drag_ghost(
    mut commands: Commands,
    drag_state: Option<Res<DragState>>,
    existing: Query<Entity, With<DragGhost>>,
) {
    let is_dragging = drag_state
        .as_ref()
        .is_some_and(|d| d.dragging.is_some());

    if !is_dragging {
        // Remove ghost if drag ended
        for entity in &existing {
            commands.entity(entity).despawn_recursive();
        }
        return;
    }

    let drag_state = drag_state.unwrap();
    let info = drag_state.dragging.as_ref().unwrap();

    // Remove old ghost and respawn at new position
    for entity in &existing {
        commands.entity(entity).despawn_recursive();
    }

    let letter = info.tile.letter().unwrap_or(' ');
    let points = info.tile.points();

    commands
        .spawn((
            DragGhost,
            Sprite {
                color: Color::srgba(0.95, 0.88, 0.7, 0.8),
                custom_size: Some(Vec2::splat(super::board::CELL_SIZE)),
                ..default()
            },
            Transform::from_xyz(info.world_pos.x, info.world_pos.y, 10.0),
            GlobalZIndex(50),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text2d::new(letter.to_string()),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.1, 0.1, 0.1)),
                Transform::from_xyz(0.0, 1.0, 0.1),
            ));
            if points > 0 {
                parent.spawn((
                    Text2d::new(points.to_string()),
                    TextFont {
                        font_size: 9.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.4, 0.4, 0.4)),
                    Transform::from_xyz(10.0, -10.0, 0.1),
                ));
            }
        });
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
                    && let Some(ref mut t) = transition
                {
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

#[allow(clippy::too_many_arguments)]
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

#[allow(clippy::too_many_arguments)]
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
        && transition.active
        && transition.is_changed()
        && overlay_query.is_empty()
    {
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
                && am.game.is_finished()
            {
                next_state.set(AppState::GameOver);
            }
        }
    }
}
