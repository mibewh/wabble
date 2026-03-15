use bevy::prelude::*;
use wabble_platform::TurnBasedGame;
use wabble_words::game::{WordGame, WordGameConfig};

use crate::app_states::AppState;
use crate::resources::{ActiveMatch, PendingPlacement, SelectedRackTile, StatusMessage, TurnTransition};

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
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
        ))
        .with_children(|parent| {
            // Title
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

            // Spacer
            parent.spawn(Node {
                height: Val::Px(40.0),
                ..default()
            });

            // Buttons
            spawn_menu_button(parent, "2 Players (Hot Seat)", MenuButton::NewGame2P);
            spawn_menu_button(parent, "3 Players (Hot Seat)", MenuButton::NewGame3P);
            spawn_menu_button(parent, "4 Players (Hot Seat)", MenuButton::NewGame4P);
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

fn menu_button_system(
    mut commands: Commands,
    interaction_query: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, button) in &interaction_query {
        if *interaction == Interaction::Pressed {
            let num_players = match button {
                MenuButton::NewGame2P => 2,
                MenuButton::NewGame3P => 3,
                MenuButton::NewGame4P => 4,
            };

            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            let config = WordGameConfig::default();
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
    }
}

fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
