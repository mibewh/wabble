use bevy::prelude::*;
use wabble_platform::TurnBasedGame;

use wabble_words::placement::validate_placement;
use wabble_words::scoring;

use crate::resources::{ActiveMatch, PendingPlacement, ScorePreview, StatusMessage};

use super::input::{PassButton, PlayButton, RecallButton};

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct StatusText;

#[derive(Component)]
pub struct CurrentPlayerText;

#[derive(Component)]
pub struct TilesRemainingText;

#[derive(Component)]
pub struct ScorePreviewPanel;

#[derive(Component)]
pub struct ScorePreviewText;

pub fn spawn_hud(mut commands: Commands) {
    commands
        .spawn((
            HudRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
        ))
        .with_children(|parent| {
            // Right panel
            parent
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(20.0),
                    top: Val::Px(20.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    padding: UiRect::all(Val::Px(15.0)),
                    ..default()
                })
                .with_children(|panel| {
                    panel.spawn((
                        CurrentPlayerText,
                        Text::new("Player 1's Turn"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    panel.spawn((
                        ScoreText,
                        Text::new("Scores: ..."),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    ));

                    panel.spawn((
                        TilesRemainingText,
                        Text::new("Bag: 86"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                    ));
                });

            // Action buttons (bottom right)
            parent
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(20.0),
                    bottom: Val::Px(20.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|panel| {
                    spawn_action_button(panel, "Play", PlayButton);
                    spawn_action_button(panel, "Pass", PassButton);
                    spawn_action_button(panel, "Recall", RecallButton);
                });

            // Score preview (left side)
            parent
                .spawn((
                    ScorePreviewPanel,
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(20.0),
                        top: Val::Px(20.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        padding: UiRect::all(Val::Px(12.0)),
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        display: Display::None,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.85)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        ScorePreviewText,
                        Text::new(""),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.95, 0.8)),
                    ));
                });

            // Status message (bottom center)
            parent
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(20.0),
                    left: Val::Px(20.0),
                    ..default()
                })
                .with_children(|panel| {
                    panel.spawn((
                        StatusText,
                        Text::new(""),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.5)),
                    ));
                });
        });
}

fn spawn_action_button(parent: &mut ChildSpawnerCommands, label: &str, marker: impl Component) {
    parent
        .spawn((
            marker,
            Button,
            Node {
                width: Val::Px(120.0),
                height: Val::Px(40.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.25, 0.25, 0.3)),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

type ScoreQuery = (With<ScoreText>, Without<CurrentPlayerText>, Without<TilesRemainingText>);
type PlayerQuery = (With<CurrentPlayerText>, Without<ScoreText>, Without<TilesRemainingText>);
type BagQuery = (With<TilesRemainingText>, Without<ScoreText>, Without<CurrentPlayerText>);

pub fn update_score_display(
    active_match: Option<Res<ActiveMatch>>,
    mut score_query: Query<&mut Text, ScoreQuery>,
    mut player_query: Query<&mut Text, PlayerQuery>,
    mut bag_query: Query<&mut Text, BagQuery>,
) {
    let Some(active_match) = active_match else {
        return;
    };
    if !active_match.is_changed() {
        return;
    }

    let state = active_match.game.state();

    // Update scores
    for mut text in &mut score_query {
        let scores: Vec<String> = state
            .players
            .iter()
            .enumerate()
            .map(|(i, p)| format!("P{}: {}", i + 1, p.score))
            .collect();
        **text = format!("Scores: {}", scores.join("  "));
    }

    // Update current player
    for mut text in &mut player_query {
        **text = format!("Player {}'s Turn", state.current_player_idx + 1);
    }

    // Update bag count
    for mut text in &mut bag_query {
        **text = format!("Bag: {} tiles", state.bag.remaining());
    }
}

pub fn update_status_display(
    status: Option<Res<StatusMessage>>,
    mut query: Query<&mut Text, With<StatusText>>,
) {
    let Some(status) = status else { return };
    if !status.is_changed() {
        return;
    }
    for mut text in &mut query {
        **text = status.text.clone();
    }
}

pub fn update_score_preview(
    active_match: Option<Res<ActiveMatch>>,
    pending: Option<Res<PendingPlacement>>,
    mut preview: ResMut<ScorePreview>,
) {
    let Some(pending) = pending else {
        *preview = ScorePreview::default();
        return;
    };
    if !pending.is_changed() {
        return;
    }

    let Some(active_match) = active_match else {
        *preview = ScorePreview::default();
        return;
    };

    if pending.tiles.is_empty() {
        *preview = ScorePreview::default();
        return;
    }

    let state = active_match.game.state();
    let placed = pending.to_placed_tiles();
    let is_first_move = state.board.is_empty();

    match validate_placement(&state.board, &placed, is_first_move) {
        Ok(validated) => {
            let tiles_map: Vec<(usize, usize, wabble_words::tile::Tile)> = placed
                .iter()
                .map(|pt| (pt.row, pt.col, pt.tile))
                .collect();

            let mut words = Vec::new();
            let mut total = 0i32;

            for (positions, word_str) in validated.words_formed.iter().zip(&validated.word_strings) {
                let word_score =
                    scoring::score_word(positions, &state.board, &placed, &tiles_map);
                total += word_score;
                words.push((word_str.clone(), word_score));
            }

            total += scoring::bingo_bonus(placed.len());

            *preview = ScorePreview {
                words,
                total_score: total,
                valid: true,
            };
        }
        Err(_) => {
            *preview = ScorePreview::default();
        }
    }
}

pub fn update_preview_display(
    preview: Option<Res<ScorePreview>>,
    mut panel_query: Query<&mut Node, With<ScorePreviewPanel>>,
    mut text_query: Query<&mut Text, With<ScorePreviewText>>,
) {
    let Some(preview) = preview else { return };
    if !preview.is_changed() {
        return;
    }

    for mut node in &mut panel_query {
        node.display = if preview.valid {
            Display::Flex
        } else {
            Display::None
        };
    }

    if preview.valid {
        let mut lines = Vec::new();
        for (word, score) in &preview.words {
            lines.push(format!("{word}  +{score}"));
        }
        if preview.words.len() > 1 || scoring::bingo_bonus(0) > 0 {
            // Always show total when multiple words
            lines.push(format!("──────\nTotal  +{}", preview.total_score));
        }
        for mut text in &mut text_query {
            **text = lines.join("\n");
        }
    }
}

type ActionButtonFilter = (With<Button>, Or<(With<PlayButton>, With<PassButton>, With<RecallButton>)>);

pub fn update_button_colors(
    mut query: Query<(&Interaction, &mut BackgroundColor), ActionButtonFilter>,
) {
    for (interaction, mut bg) in &mut query {
        *bg = match interaction {
            Interaction::Pressed => BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
            Interaction::Hovered => BackgroundColor(Color::srgb(0.35, 0.35, 0.45)),
            Interaction::None => BackgroundColor(Color::srgb(0.25, 0.25, 0.3)),
        };
    }
}

pub fn cleanup_hud(mut commands: Commands, query: Query<Entity, With<HudRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
