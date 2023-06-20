use bevy::ecs::schedule::StateData;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::log;
use bevy::utils::AHashExt;
use bevy::utils::HashMap;
use components::Bomb;
use components::BombNeighbor;
use resources::BoardOptions;
use resources::tile::Tile;
use resources::tile_map::TileMap;

#[cfg(feature = "debug")]
use bevy_inspector_egui::RegisterInspectable;

use crate::bounds::Bounds2;
use crate::components::Coordinates;
use crate::components::Uncover;
use crate::events::TileTriggerEvent;
use crate::resources::Board;
use crate::resources::BoardPosition;
use crate::resources::TileSize;

pub mod components;
pub mod resources;
mod bounds;
mod systems;
mod events;

pub struct BoardPlugin<T> {
    pub running_state: T
}

impl<T: StateData> Plugin for BoardPlugin<T> {
    fn build(&self, app: &mut App) {
        // When the running states comes into the stack we load a board
        app.add_system_set(
            SystemSet::on_enter(self.running_state.clone()).with_system(Self::create_board),
        )
        // We handle input and trigger events only if the state is active
        .add_system_set(
            SystemSet::on_update(self.running_state.clone())
                .with_system(systems::input::input_handling)
                .with_system(systems::uncover::trigger_event_handler),
        )
        // We handle uncovering even if the state is inactive
        .add_system_set(
            SystemSet::on_in_stack_update(self.running_state.clone())
                .with_system(systems::uncover::uncover_tiles),
        )
        .add_system_set(
            SystemSet::on_exit(self.running_state.clone())
                .with_system(Self::cleanup_board),
        )
        .add_event::<TileTriggerEvent>();

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

impl<T> BoardPlugin<T> {
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

        let mut covered_tiles = 
            HashMap::with_capacity((tile_map.width() * tile_map.height()).into());

        let mut safe_start = None;

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

        let board_entity = commands.spawn()
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
                        Color::DARK_GRAY,
                        &mut covered_tiles,
                        &mut safe_start
                    );
                })
                .id();

        commands.insert_resource(Board {
            tile_map,
            bounds: Bounds2 {
                position: board_position.xy(),
                size: board_size
            },
            tile_size,
            covered_tiles,
            entity: board_entity,
        });

        if options.safe_start {
            if let Some(entity) = safe_start {
                commands.entity(entity).insert(Uncover);
            }
        }
    }

    fn spawn_tiles(
        parent: &mut ChildBuilder,
        tile_map: &TileMap,
        size: f32,
        padding: f32,
        color: Color,
        bomb_image: Handle<Image>,
        font: Handle<Font>,
        covered_tile_color: Color,
        covered_tiles: &mut HashMap<Coordinates, Entity>,
        safe_start_entity: &mut Option<Entity>,
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

                // We add the cover sprites
                cmd.with_children(|parent| {
                    let entity = parent.spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            custom_size: Some(Vec2::splat(size - padding)),
                            color: covered_tile_color,
                            ..Default::default()
                        },
                        transform: Transform::from_xyz(0.0, 0.0, 2.0),
                        ..Default::default()
                    })
                    .insert(Name::new("Tile Cover"))
                    .id();
                    covered_tiles.insert(Coordinates {
                        x: x as u16,
                        y: y as u16
                    }, entity);
                    if safe_start_entity.is_none() && *tile == Tile::Empty {
                        *safe_start_entity = Some(entity);
                    }
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

    fn cleanup_board(board: Res<Board>, mut commands: Commands) {
        commands.entity(board.entity).despawn_recursive();
        commands.remove_resource::<Board>();
    }
}