use bevy::{prelude::*, log};

use crate::{resources::{Board, BoardAssets}, events::TileMarkEvent};

pub fn mark_tiles(
    mut commands: Commands,
    mut board: ResMut<Board>,
    board_assets: Res<BoardAssets>,
    mut tile_mark: EventReader<TileMarkEvent>,
    query: Query<&Children>,
) {
    for event in tile_mark.iter() {
        if let Some((entity, mark)) = board.try_toggle_mark(&event.0) {
            if mark {
                commands.entity(entity).with_children(|parent| {
                    parent.spawn_bundle(SpriteBundle {
                        texture: board_assets.flag_material.texture.clone(),
                        sprite: Sprite {
                            custom_size: Some(Vec2::splat(board.tile_size)),
                            color: board_assets.flag_material.color,
                            ..Default::default()
                        },
                        transform: Transform::from_xyz(0.0, 0.0, 1.0),
                        ..Default::default()
                    })
                          .insert(Name::new("Flag"));
                });
            } else {
                let children = match query.get(entity) {
                    Ok(c) => c,
                    Err(e) => {
                        log::error!("Failed to retrieve flag entity components: {}", e);
                        continue;
                    }
                };
                for child in children.iter() {
                    commands.entity(*child).despawn_recursive();
                }
            }
        }
    }
}