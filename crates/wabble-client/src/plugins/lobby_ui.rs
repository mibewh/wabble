use std::sync::Arc;

use bevy::prelude::*;
use wabble_platform::TurnBasedGame;
use wabble_words::game::{WordGame, WordGameConfig};

use crate::app_states::AppState;
use crate::resources::{
    ActiveMatch, AiOpponent, PendingPlacement, SelectedRackTile, StatusMessage, TurnTransition,
};

pub struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), setup_menu)
            .add_systems(
                Update,
                menu_button_system.run_if(in_state(AppState::MainMenu)),
            )
            .add_systems(OnExit(AppState::MainMenu), cleanup_menu);
    }
}

#[derive(Component)]
struct MenuRoot;

#[derive(Component)]
enum MenuButton {
    NewGame2P,
    NewGame3P,
    NewGame4P,
    VsAiEasy,
    VsAiMedium,
    VsAiHard,
}

fn setup_menu(mut commands: Commands) {
    commands
        .spawn((
            MenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("WABBLE"),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.75, 0.3)),
            ));

            parent.spawn((
                Text::new("A Word Game"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));

            parent.spawn(Node {
                height: Val::Px(20.0),
                ..default()
            });

            // Hot seat section
            parent.spawn((
                Text::new("Hot Seat"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
            spawn_menu_button(parent, "2 Players", MenuButton::NewGame2P);
            spawn_menu_button(parent, "3 Players", MenuButton::NewGame3P);
            spawn_menu_button(parent, "4 Players", MenuButton::NewGame4P);

            parent.spawn(Node {
                height: Val::Px(10.0),
                ..default()
            });

            // VS AI section
            parent.spawn((
                Text::new("vs AI"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
            spawn_menu_button(parent, "Easy AI", MenuButton::VsAiEasy);
            spawn_menu_button(parent, "Medium AI", MenuButton::VsAiMedium);
            spawn_menu_button(parent, "Hard AI", MenuButton::VsAiHard);
        });
}

fn spawn_menu_button(parent: &mut ChildBuilder, label: &str, button: MenuButton) {
    parent
        .spawn((
            button,
            Button,
            Node {
                width: Val::Px(300.0),
                height: Val::Px(50.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.25, 0.25, 0.3)),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn try_load_ai(
    difficulty: wabble_ai::difficulty::Difficulty,
) -> Option<Arc<wabble_ai::WordGameAi>> {
    let gaddag_bytes = std::fs::read("assets/gaddag.fst").ok()?;
    let dict_bytes = std::fs::read("assets/dict.fst").ok()?;
    let gaddag = wabble_dict::Gaddag::from_bytes(gaddag_bytes).ok()?;
    let dict = wabble_dict::FstDictionary::from_bytes(dict_bytes).ok()?;
    Some(Arc::new(wabble_ai::WordGameAi::new(
        gaddag, dict, difficulty,
    )))
}

fn menu_button_system(
    mut commands: Commands,
    interaction_query: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, button) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let config = WordGameConfig::default();

        match button {
            MenuButton::NewGame2P | MenuButton::NewGame3P | MenuButton::NewGame4P => {
                let num_players = match button {
                    MenuButton::NewGame2P => 2,
                    MenuButton::NewGame3P => 3,
                    MenuButton::NewGame4P => 4,
                    _ => unreachable!(),
                };
                let game = WordGame::new_game(&config, num_players, seed);
                commands.insert_resource(ActiveMatch {
                    game,
                    player_count: num_players,
                });
                commands.insert_resource(PendingPlacement::default());
                commands.insert_resource(SelectedRackTile::default());
                commands.insert_resource(TurnTransition::default());
                commands.insert_resource(StatusMessage::default());
                next_state.set(AppState::InGame);
            }
            MenuButton::VsAiEasy | MenuButton::VsAiMedium | MenuButton::VsAiHard => {
                let difficulty = match button {
                    MenuButton::VsAiEasy => wabble_ai::difficulty::Difficulty::Easy,
                    MenuButton::VsAiMedium => wabble_ai::difficulty::Difficulty::Medium,
                    MenuButton::VsAiHard => wabble_ai::difficulty::Difficulty::Hard,
                    _ => unreachable!(),
                };

                let ai = match try_load_ai(difficulty) {
                    Some(ai) => ai,
                    None => {
                        warn!("Failed to load AI dictionary files (assets/dict.fst, assets/gaddag.fst)");
                        continue;
                    }
                };

                let game = WordGame::new_game(&config, 2, seed);
                commands.insert_resource(ActiveMatch {
                    game,
                    player_count: 2,
                });
                commands.insert_resource(AiOpponent {
                    player_idx: 1,
                    ai,
                });
                commands.insert_resource(PendingPlacement::default());
                commands.insert_resource(SelectedRackTile::default());
                commands.insert_resource(TurnTransition::default());
                commands.insert_resource(StatusMessage::default());
                next_state.set(AppState::InGame);
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
