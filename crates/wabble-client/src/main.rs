#![allow(dead_code)]

mod app_states;
mod plugins;
mod resources;

use bevy::prelude::*;

use app_states::AppState;
use plugins::audio::AudioPlugin as WabbleAudioPlugin;
use plugins::lobby_ui::LobbyPlugin;
use plugins::network::NetworkPlugin;
use plugins::profile_ui::ProfilePlugin;
use plugins::shell::ShellPlugin;
use plugins::words::WordsGamePlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Wabble".to_string(),
                    resolution: bevy::window::WindowResolution::new(1100, 750),
                    ..default()
                }),
                ..default()
            }),
        )
        .init_state::<AppState>()
        .add_plugins((
            ShellPlugin,
            LobbyPlugin,
            ProfilePlugin,
            NetworkPlugin,
            WabbleAudioPlugin,
            WordsGamePlugin,
        ))
        .add_systems(OnEnter(AppState::GameOver), setup_game_over)
        .add_systems(
            Update,
            game_over_button.run_if(in_state(AppState::GameOver)),
        )
        .add_systems(OnExit(AppState::GameOver), cleanup_game_over)
        .run();
}

#[derive(Component)]
struct GameOverRoot;

#[derive(Component)]
struct NewGameButton;

fn setup_game_over(mut commands: Commands, active_match: Option<Res<resources::ActiveMatch>>) {
    let results_text = if let Some(am) = active_match {
        use wabble_platform::TurnBasedGame;
        if let Some(results) = am.game.results() {
            let scores: Vec<String> = results
                .player_scores
                .iter()
                .enumerate()
                .map(|(i, s)| format!("Player {}: {s}", i + 1))
                .collect();
            let winner = match results.winner {
                Some(w) => format!("Player {} wins!", w + 1),
                None => "It's a draw!".to_string(),
            };
            format!("{winner}\n\n{}", scores.join("\n"))
        } else {
            "Game Over".to_string()
        }
    } else {
        "Game Over".to_string()
    };

    commands
        .spawn((
            GameOverRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Game Over"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.75, 0.3)),
            ));

            parent.spawn((
                Text::new(results_text),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn(Node {
                height: Val::Px(20.0),
                ..default()
            });

            parent
                .spawn((
                    NewGameButton,
                    Button,
                    Node {
                        width: Val::Px(250.0),
                        height: Val::Px(50.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.25, 0.25, 0.3)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Back to Menu"),
                        TextFont {
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

fn game_over_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<NewGameButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            commands.remove_resource::<resources::ActiveMatch>();
            commands.remove_resource::<resources::PendingPlacement>();
            commands.remove_resource::<resources::SelectedRackTile>();
            commands.remove_resource::<resources::DragState>();
            commands.remove_resource::<resources::TurnTransition>();
            commands.remove_resource::<resources::StatusMessage>();
            commands.remove_resource::<resources::AiOpponent>();
            commands.remove_resource::<resources::AiMoveTimer>();
            next_state.set(AppState::MainMenu);
        }
    }
}

fn cleanup_game_over(mut commands: Commands, query: Query<Entity, With<GameOverRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
