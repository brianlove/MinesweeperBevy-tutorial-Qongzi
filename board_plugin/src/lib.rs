pub mod components;
pub mod resources;
mod bounds;
mod events;
mod systems;

use bevy::utils::HashMap;
use bevy::log;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
#[cfg(feature = "debug")]
use bevy_inspector_egui::RegisterInspectable;
use bounds::Bounds2;
use components::*;
use crate::events::*;
use crate::resources::tile::Tile;
use resources::tile_map::TileMap;
use resources::Board;
use resources::BoardOptions;
use resources::BoardPosition;
use resources::TileSize;


pub struct BoardPlugin;


impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(Self::create_board)
            .add_system(systems::input::input_handling)
            .add_system(systems::uncover::trigger_event_handler)
            .add_system(systems::uncover::uncover_tiles)
            .add_event::<TileTriggerEvent>();
        log::info!("Loaded BoardPlugin");

        #[cfg(feature = "debug")]
        {
            // Lets us edit components in the inspector
            app.register_inspectable::<Coordinates>();
            app.register_inspectable::<BombNeighbor>();
            app.register_inspectable::<Bomb>();
            app.register_inspectable::<Uncover>();
        }
    }
}

impl BoardPlugin {
    /// System to generate the complete board
    pub fn create_board(
        mut commands: Commands,
        board_options: Option<Res<BoardOptions>>,
        // ISSUE: `window` isn't working (likely due to Bevy 0.9)
        // window: Res<WindowDescriptor>,
        windows: Res<Windows>,
        mut materials: ResMut<Assets<ColorMaterial>>,
        asset_server: Res<AssetServer>,
    ) {
        let options = match board_options {
            None => BoardOptions::default(),
            Some(o) => o.clone(),
        };

        let mut tile_map = TileMap::empty(options.map_size.0, options.map_size.1);
        tile_map.set_bombs(options.bomb_count);
        #[cfg(feature = "debug")]
        log::info!("{}", tile_map.console_output());

        let tile_size = match options.tile_size {
            TileSize::Fixed(v) => v,
            TileSize::Adaptive {min, max} => Self::adaptive_tile_size(
                windows.get_primary().unwrap(),
                (min, max),
                (tile_map.width(), tile_map.height()),
            ),
        };

        let board_size = Vec2::new(
            tile_map.width() as f32 * tile_size,
            tile_map.height() as f32 * tile_size,
        );
        log::info!("board size: {}", board_size);

        let board_position = match options.position {
            BoardPosition::Centered { offset } => {
                Vec3::new(-(board_size.x / 2.), -(board_size.y / 2.), 0.) + offset
            }
            BoardPosition::Custom(p) => p,
        };

        // TODO: refactor into a resource later on (in later chapter)
        let font: Handle<Font> = asset_server.load("fonts/pixeled.ttf");
        let bomb_image: Handle<Image> = asset_server.load("sprites/bomb.png");

        let mut covered_tiles = HashMap::with_capacity((tile_map.width() * tile_map.height()).into());
        let mut safe_start = None;

        commands
            // NOTE: Bevy 0.9 expects a `Bundle` with `.spawn()`, but `.spawn_empty()` is available instead
            // Actually, we need to use `.spawn(SpatialBundle::default())` to make sure that things have visibility
            // https://bevyengine.org/learn/book/migration-guides/0.7-0.8/#visibilty-inheritance-universal-computedvisibility-and-renderlayers-support
            .spawn(SpatialBundle::default())
            .insert(Name::new("Board"))
            .insert(Transform::from_translation(board_position))
            .insert(GlobalTransform::default())
            .with_children(|parent| {
                parent
                    // NOTE: Bevy 0.9 uses `.spawn()`, which can now handle `Bundle`s
                    .spawn(SpriteBundle {
                        sprite: Sprite {
                            color: Color::WHITE,
                            custom_size: Some(board_size),
                            ..Default::default()
                        },
                        transform: Transform::from_xyz(board_size.x / 2., board_size.y / 2., 0.),
                        ..Default::default()
                    })
                    .insert(Name::new("Background"));

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
                    &mut safe_start,
                );
            });

        if options.safe_start {
            if let Some(entity) = safe_start {
                commands.entity(entity).insert(Uncover {});
            }
        }

        commands
            .insert_resource(Board {
                bounds: Bounds2 {
                    position: board_position.xy(),
                    size: board_size,
                },
                covered_tiles,
                tile_map,
                tile_size,
            });
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
        for (y, line) in tile_map.iter().enumerate() {
            for (x, tile) in line.iter().enumerate() {
                let coordinates = Coordinates {
                    x: x as u16,
                    y: y as u16,
                };
                // let mut cmd = parent.spawn_empty();
                let mut cmd = parent.spawn(SpatialBundle::default());
                cmd.insert(SpriteBundle {
                        sprite: Sprite {
                            color: Color::GRAY,
                            custom_size: Some(Vec2::splat(
                                size - padding as f32,
                            )),
                            ..Default::default()
                        },
                        transform: Transform::from_xyz(
                            (x as f32 * size) + (size / 2.),
                            (y as f32 * size) + (size / 2.),
                            1.,
                        ),
                        ..Default::default()
                    })
                    .insert(Name::new(format!("Tile ({}, {})", x, y)))
                    .insert(Coordinates {
                        x: x as u16,
                        y: y as u16,
                    });

                match tile {
                    Tile::Bomb => {
                        cmd.insert(Bomb);
                        cmd.with_children(|parent| {
                            parent.spawn(SpriteBundle {
                                sprite: Sprite {
                                    custom_size: Some(Vec2::splat(size - padding)),
                                    ..Default::default()
                                },
                                transform: Transform::from_xyz(0., 0., 1.),
                                texture: bomb_image.clone(),
                                ..Default::default()
                            });
                        });
                    },
                    Tile::BombNeighbor(v) => {
                        cmd.insert(BombNeighbor { count: *v });
                        cmd.with_children(|parent| {
                            parent.spawn(Self::bomb_count_text_bundle(
                                *v,
                                font.clone(),
                                size - padding,
                            ));
                        });
                    },
                    Tile::Empty => (),
                }

                // Add the cover sprites
                cmd.with_children(|parent| {
                    let entity = parent
                        .spawn(SpriteBundle {
                            sprite: Sprite {
                                custom_size: Some(Vec2::splat(size - padding)),
                                color: covered_tile_color,
                                ..Default::default()
                            },
                            transform: Transform::from_xyz(0., 0., 2.),
                            ..Default::default()
                        })
                        .insert(Name::new("Tile cover"))
                        .id();
                    covered_tiles.insert(coordinates, entity);
                    if safe_start_entity.is_none() && *tile == Tile::Empty {
                        *safe_start_entity = Some(entity);
                    }
                });
            }
        }
    }


    fn bomb_count_text_bundle(count: u8, font: Handle<Font>, size: f32) -> Text2dBundle {
        let (text, color) = (
            count.to_string(),
            match count {
                1 => Color::WHITE,
                2 => Color::GREEN,
                3 => Color::YELLOW,
                4 => Color::ORANGE,
                _ => Color::PURPLE,
            },
        );

        return Text2dBundle {
            text: Text {
                sections: vec![TextSection {
                    value: text,
                    style: TextStyle {
                        color,
                        font,
                        font_size: size,
                    },
                }],
                alignment: TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Center,
                },
            },
            transform: Transform::from_xyz(0., 0., 1.),
            ..Default::default()
        };
    }

    fn adaptive_tile_size(
        window: &Window,
        (min, max): (f32, f32),
        (width, height): (u16, u16),
    ) -> f32 {
        let max_width = window.width() / width as f32;
        let max_height = window.height() / height as f32;
        return max_width.min(max_height).clamp(min, max);
    }
}


pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
