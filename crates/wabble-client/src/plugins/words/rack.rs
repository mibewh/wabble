use bevy::prelude::*;
use wabble_platform::TurnBasedGame;

use crate::resources::{ActiveMatch, PendingPlacement, SelectedRackTile, TurnTransition};

pub const RACK_Y: f32 = -280.0;
pub const RACK_TILE_SIZE: f32 = 40.0;
pub const RACK_STRIDE: f32 = 46.0;

#[derive(Component)]
pub struct RackRoot;

#[derive(Component)]
pub struct RackTileSprite {
    pub index: usize,
}

/// Convert rack slot index to world position.
pub fn rack_slot_pos(index: usize, total: usize) -> Vec2 {
    let offset = (total as f32 - 1.0) / 2.0;
    let x = (index as f32 - offset) * RACK_STRIDE;
    Vec2::new(x, RACK_Y)
}

/// Check if a world position hits a rack tile.
pub fn world_to_rack_index(pos: Vec2, total: usize) -> Option<usize> {
    for i in 0..total {
        let slot = rack_slot_pos(i, total);
        let dist = (pos - slot).length();
        if dist < RACK_TILE_SIZE / 2.0 {
            return Some(i);
        }
    }
    None
}

pub fn spawn_rack(mut commands: Commands) {
    commands.spawn((RackRoot, Transform::default(), Visibility::default()));
}

pub fn update_rack_display(
    mut commands: Commands,
    active_match: Option<Res<ActiveMatch>>,
    selected: Option<Res<SelectedRackTile>>,
    pending: Option<Res<PendingPlacement>>,
    transition: Option<Res<TurnTransition>>,
    existing: Query<Entity, With<RackTileSprite>>,
) {
    let Some(active_match) = active_match else {
        return;
    };

    // Check if anything relevant changed
    let match_changed = active_match.is_changed();
    let selected_changed = selected.as_ref().is_some_and(|s| s.is_changed());
    let pending_changed = pending.as_ref().is_some_and(|p| p.is_changed());
    let transition_changed = transition.as_ref().is_some_and(|t| t.is_changed());

    if !match_changed && !selected_changed && !pending_changed && !transition_changed {
        return;
    }

    // If in turn transition, hide rack
    if let Some(ref t) = transition
        && t.active {
            for entity in &existing {
                commands.entity(entity).despawn_recursive();
            }
            return;
        }

    // Remove old rack sprites
    for entity in &existing {
        commands.entity(entity).despawn_recursive();
    }

    let current_player = active_match.game.state().current_player_idx;
    let player_state = match active_match.game.player_state(current_player) {
        Some(ps) => ps,
        None => return,
    };

    // Figure out which rack indices are currently pending on the board
    let pending_indices: Vec<usize> = pending
        .as_ref()
        .map(|p| p.tiles.iter().map(|t| t.rack_index).collect())
        .unwrap_or_default();

    let selected_idx = selected.as_ref().and_then(|s| s.index);
    let rack = &player_state.rack;
    let total_visible = rack.len();

    for (i, tile) in rack.tiles.iter().enumerate() {
        if pending_indices.contains(&i) {
            continue; // This tile is placed on the board
        }

        let pos = rack_slot_pos(i, total_visible);
        let is_selected = selected_idx == Some(i);

        let bg_color = if is_selected {
            Color::srgb(0.7, 0.85, 0.7) // green highlight
        } else {
            Color::srgb(0.95, 0.88, 0.7)
        };

        let letter = tile.letter().unwrap_or(' ');
        let points = tile.points();

        commands
            .spawn((
                RackTileSprite { index: i },
                Sprite {
                    color: bg_color,
                    custom_size: Some(Vec2::splat(RACK_TILE_SIZE)),
                    ..default()
                },
                Transform::from_xyz(pos.x, pos.y, 1.0),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text2d::new(letter.to_string()),
                    TextFont {
                        font_size: 26.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.1, 0.1, 0.1)),
                    Transform::from_xyz(0.0, 1.0, 0.1),
                ));

                if points > 0 {
                    parent.spawn((
                        Text2d::new(points.to_string()),
                        TextFont {
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.4, 0.4, 0.4)),
                        Transform::from_xyz(12.0, -12.0, 0.1),
                    ));
                }
            });
    }
}

pub fn cleanup_rack(mut commands: Commands, query: Query<Entity, With<RackRoot>>, tiles: Query<Entity, With<RackTileSprite>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    for entity in &tiles {
        commands.entity(entity).despawn_recursive();
    }
}
