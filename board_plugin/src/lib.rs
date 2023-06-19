use std::ops::Deref;

use bevy::prelude::*;
use bevy::log;
use components::Bomb;
use components::BombNeighbor;
use resources::BoardOptions;
use resources::tile::Tile;
use resources::tile_map::TileMap;

use crate::components::Coordinates;
use crate::resources::BoardPosition;
use crate::resources::TileSize;

pub mod components;
pub mod resources;

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(Self::create_board);

        #[cfg(feature = "debug")]
        {
            // registering custom component to be able to edit it in inspector
            app.register_inspectable::<Coordinates>();
            app.register_inspectable::<BombNeighbor>();
            app.register_inspectable::<Bomb>();
            app.register_inspectable::<Uncover>();
        }

        log::info!("Loaded Board Plugin");
    }
}

impl BoardPlugin {
    pub fn create_board(
        mut commands: Commands,
        board_options: Option<Res<BoardOptions>>,
        window: Option<Res<WindowDescriptor>>,
        asset_server: Res<AssetServer>
    ) {
        let font: Handle<Font> = asset_server.load("fonts/pixeled.ttf");
        let bomb_image: Handle<Image> = asset_server.load("sprites/bomb.png");
        let options = board_options
            .map(|o| o.clone())
            .unwrap_or_default();

        let mut tile_map = TileMap::empty(options.map_size.0, options.map_size.1);
        tile_map.set_bombs(options.bomb_count);

        #[cfg(feature = "debug")]
        log::info!("{}", tile_map.console_output());

        // We define the size of our tiles in world space
        let tile_size = match options.tile_size {
            TileSize::Fixed(v) => v,
            TileSize::Adaptive { min, max } => 
                Self::adaptive_tile_size(
                    window, (min, max), (tile_map.width(), tile_map.height())
                )
        };

        let board_size = Vec2::new(
            tile_map.width() as f32 * tile_size,
            tile_map.height() as f32 * tile_size,
        );
        log::info!("board size: {}", board_size);

        // We define the board anchor position (bottom left)
        let board_position = match options.position {
            BoardPosition::Centered { offset } => {
                Vec3::new(-(board_size.x / 2.0), -(board_size.y / 2.0), 0.0) + offset
            },
            BoardPosition::Custom(p) => p
        };

        commands.spawn()
                .with_children(|parent| {
                    // We spawn the board background sprite at the center of the board, since the sprite pivot is centered
                    parent.spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            color: Color::WHITE,
                            custom_size: Some(board_size),
                            ..Default::default()
                        },
                        transform: Transform::from_xyz(board_size.x / 2.0, board_size.y / 2.0, 0.0),
                        ..Default::default()
                    })
                          .insert(Name::new("Background"));
                })
                .insert(Name::new("Board"))
                .insert(Transform::from_translation(board_position))
                .insert(GlobalTransform::default())
                .with_children(|parent| {
                    Self::spawn_tiles(
                        parent,
                        &tile_map,
                        tile_size,
                        options.tile_padding,
                        Color::GRAY,
                        bomb_image,
                        font,
                    );
                });
    }

    fn spawn_tiles(
        parent: &mut ChildBuilder,
        tile_map: &TileMap,
        size: f32,
        padding: f32,
        color: Color,
        bomb_image: Handle<Image>,
        font: Handle<Font>
    ) {
        // Tiles
        for (y, line) in tile_map.iter().enumerate() {
            for (x, tile) in line.iter().enumerate() {
                let mut cmd = parent.spawn();
                cmd.insert_bundle(SpriteBundle {
                    sprite: Sprite {
                        color,
                        custom_size: Some(Vec2::splat(size - padding)),
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(
                        (x as f32 * size) + (size / 2.0),
                        (y as f32 * size) + (size / 2.0),
                        1.0
                    ),
                    ..Default::default()
                })
                .insert(Name::new(format!("Tile ({}, {})", x, y)))
                .insert(Coordinates {
                    x: x as u16,
                    y: y as u16
                });

                match tile {
                    // If the tile is a bomb we add the matching component and a sprite child
                    Tile::Bomb => {
                        cmd.insert(Bomb);
                        cmd.with_children(|parent| {
                            parent.spawn_bundle(SpriteBundle {
                                sprite: Sprite {
                                    custom_size: Some(Vec2::splat(size - padding)),
                                    ..Default::default()
                                },
                                transform: Transform::from_xyz(0.0, 0.0, 1.0),
                                texture: bomb_image.clone(),
                                ..Default::default()
                            });
                        });
                    },
                    // If the tile is a bomb neighbour we add the matching component and a text child
                    Tile::BombNeighbor(v) => {
                        cmd.insert(BombNeighbor { count: *v });
                        cmd.with_children(|parent| {
                            parent.spawn_bundle(Self::bomb_count_text_bundle(
                                *v,
                                font.clone(),
                                size - padding
                            ));
                        });
                    }
                    Tile::Empty => ()
                }
            }
        }
    }

    /// Computes a tile size that matches the window according to the tile map size
    fn adaptive_tile_size(
        window: Option<Res<WindowDescriptor>>,
        (min, max): (f32, f32),
        (width, height): (u16, u16)
    ) -> f32 {
        let max_width = window.as_ref().map(|w| w.as_ref().width).unwrap_or(1280.0) / width as f32;
        let max_height = window.map(|w| w.height).unwrap_or(720.0) / height as f32;

        max_width.min(max_height).clamp(min, max)
    }

    /// Generates the bomb counter text 2D Bundle for a given value
    fn bomb_count_text_bundle(count: u8, font: Handle<Font>, size: f32) -> Text2dBundle {
        let (text, color) = (
            count.to_string(),
            match count {
                1 => Color::WHITE,
                2 => Color::GREEN,
                3 => Color::YELLOW,
                4 => Color::ORANGE,
                _ => Color::PURPLE
            }
        );

        Text2dBundle {
            text: Text {
                sections: vec![TextSection {
                    value: text,
                    style: TextStyle {
                        font,
                        color,
                        font_size: size
                    }
                }],
                alignment: TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Center
                }
            },
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            ..Default::default()
        }
    }
}