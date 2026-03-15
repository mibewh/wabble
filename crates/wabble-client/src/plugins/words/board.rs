use bevy::prelude::*;
use wabble_platform::TurnBasedGame;
use wabble_words::board::{BOARD_SIZE, Board, BonusSquare};
use wabble_words::tile::Tile;

use crate::resources::{ActiveMatch, PendingPlacement};

pub const CELL_SIZE: f32 = 34.0;
pub const CELL_STRIDE: f32 = 36.0;
pub const BOARD_Y_OFFSET: f32 = 50.0;

#[derive(Component)]
pub struct BoardRoot;

#[derive(Component)]
pub struct BoardCell {
    pub row: usize,
    pub col: usize,
}

#[derive(Component)]
pub struct BoardTileSprite {
    pub row: usize,
    pub col: usize,
}

#[derive(Component)]
pub struct PendingTileSprite {
    pub row: usize,
    pub col: usize,
}

/// Convert board grid position to world coordinates.
pub fn grid_to_world(row: usize, col: usize) -> Vec2 {
    let x = (col as f32 - 7.0) * CELL_STRIDE;
    let y = (7.0 - row as f32) * CELL_STRIDE + BOARD_Y_OFFSET;
    Vec2::new(x, y)
}

/// Convert world position to board grid position, if within bounds.
pub fn world_to_grid(pos: Vec2) -> Option<(usize, usize)> {
    let col_f = (pos.x / CELL_STRIDE) + 7.0;
    let row_f = 7.0 - ((pos.y - BOARD_Y_OFFSET) / CELL_STRIDE);

    let col = col_f.round() as i32;
    let row = row_f.round() as i32;

    if col >= 0
        && col < BOARD_SIZE as i32
        && row >= 0
        && row < BOARD_SIZE as i32
    {
        // Check we're actually close to the cell center
        let center = grid_to_world(row as usize, col as usize);
        let dist = (pos - center).length();
        if dist < CELL_SIZE / 2.0 {
            return Some((row as usize, col as usize));
        }
    }
    None
}

fn bonus_color(bonus: BonusSquare) -> Color {
    match bonus {
        BonusSquare::Normal => Color::srgb(0.85, 0.78, 0.68),
        BonusSquare::DoubleLetter => Color::srgb(0.6, 0.8, 0.9),
        BonusSquare::TripleLetter => Color::srgb(0.2, 0.5, 0.8),
        BonusSquare::DoubleWord => Color::srgb(0.9, 0.7, 0.7),
        BonusSquare::TripleWord => Color::srgb(0.85, 0.3, 0.25),
        BonusSquare::Center => Color::srgb(0.9, 0.7, 0.7),
    }
}

fn bonus_label(bonus: BonusSquare) -> &'static str {
    match bonus {
        BonusSquare::Normal => "",
        BonusSquare::DoubleLetter => "DL",
        BonusSquare::TripleLetter => "TL",
        BonusSquare::DoubleWord => "DW",
        BonusSquare::TripleWord => "TW",
        BonusSquare::Center => "*",
    }
}

pub fn spawn_board(mut commands: Commands) {
    commands
        .spawn((BoardRoot, Transform::default(), Visibility::default()))
        .with_children(|parent| {
            for row in 0..BOARD_SIZE {
                for col in 0..BOARD_SIZE {
                    let pos = grid_to_world(row, col);
                    let bonus = Board::bonus_at(row, col);

                    parent
                        .spawn((
                            BoardCell { row, col },
                            Sprite {
                                color: bonus_color(bonus),
                                custom_size: Some(Vec2::splat(CELL_SIZE)),
                                ..default()
                            },
                            Transform::from_xyz(pos.x, pos.y, 0.0),
                        ))
                        .with_children(|cell| {
                            let label = bonus_label(bonus);
                            if !label.is_empty() {
                                cell.spawn((
                                    Text2d::new(label),
                                    TextFont {
                                        font_size: 10.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgba(0.0, 0.0, 0.0, 0.4)),
                                    Transform::from_xyz(0.0, 0.0, 0.1),
                                ));
                            }
                        });
                }
            }
        });
}

pub fn update_board_display(
    mut commands: Commands,
    active_match: Option<Res<ActiveMatch>>,
    pending: Option<Res<PendingPlacement>>,
    existing_tiles: Query<Entity, With<BoardTileSprite>>,
    existing_pending: Query<Entity, With<PendingTileSprite>>,
) {
    let Some(active_match) = active_match else {
        return;
    };
    if !active_match.is_changed() {
        // Also check if pending changed
        if let Some(ref p) = pending {
            if !p.is_changed() {
                return;
            }
        } else {
            return;
        }
    }

    // Remove old tile sprites
    for entity in &existing_tiles {
        commands.entity(entity).despawn();
    }
    for entity in &existing_pending {
        commands.entity(entity).despawn();
    }

    let board = &active_match.game.state().board;

    // Spawn committed tiles
    for row in 0..BOARD_SIZE {
        for col in 0..BOARD_SIZE {
            if let Some(tile) = board.get(row, col) {
                spawn_tile_sprite(&mut commands, row, col, *tile, false);
            }
        }
    }

    // Spawn pending tiles
    if let Some(pending) = pending {
        for pt in &pending.tiles {
            spawn_tile_sprite(&mut commands, pt.row, pt.col, pt.tile, true);
        }
    }
}

fn spawn_tile_sprite(commands: &mut Commands, row: usize, col: usize, tile: Tile, is_pending: bool) {
    let pos = grid_to_world(row, col);
    let bg_color = if is_pending {
        Color::srgb(0.95, 0.9, 0.7)
    } else {
        Color::srgb(0.95, 0.88, 0.7)
    };

    let letter = tile.letter().unwrap_or(' ');
    let points = tile.points();

    let mut entity = commands.spawn((
        Sprite {
            color: bg_color,
            custom_size: Some(Vec2::splat(CELL_SIZE - 2.0)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 1.0),
    ));

    if is_pending {
        entity.insert(PendingTileSprite { row, col });
    } else {
        entity.insert(BoardTileSprite { row, col });
    }

    entity.with_children(|parent| {
        // Letter
        parent.spawn((
            Text2d::new(letter.to_string()),
            TextFont {
                font_size: 22.0,
                ..default()
            },
            TextColor(if tile.is_blank() {
                Color::srgb(0.6, 0.2, 0.2)
            } else {
                Color::srgb(0.1, 0.1, 0.1)
            }),
            Transform::from_xyz(0.0, 1.0, 0.1),
        ));

        // Point value (small, bottom-right)
        if points > 0 {
            parent.spawn((
                Text2d::new(points.to_string()),
                TextFont {
                    font_size: 9.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 0.3, 0.3)),
                Transform::from_xyz(10.0, -10.0, 0.1),
            ));
        }
    });
}

pub fn cleanup_board(
    mut commands: Commands,
    board_root: Query<Entity, With<BoardRoot>>,
    tile_sprites: Query<Entity, With<BoardTileSprite>>,
    pending_sprites: Query<Entity, With<PendingTileSprite>>,
) {
    for entity in &board_root {
        commands.entity(entity).despawn();
    }
    for entity in &tile_sprites {
        commands.entity(entity).despawn();
    }
    for entity in &pending_sprites {
        commands.entity(entity).despawn();
    }
}
