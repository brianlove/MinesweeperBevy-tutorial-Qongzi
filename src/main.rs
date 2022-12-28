use bevy::prelude::*;
use board_plugin::BoardPlugin;
use board_plugin::resources::BoardOptions;

#[cfg(feature = "debug")]
use bevy_inspector_egui::WorldInspectorPlugin;


fn camera_setup(mut commands: Commands) {
    // 2D orthographic camera
    commands.spawn(Camera2dBundle::default());
}

fn main() {
    let mut app = App::new();

    // Window setup
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        window: WindowDescriptor {
            title: "Minesweeper".to_string(),
            width: 700.,
            height: 800.,
            ..default()
        },
        ..default()
    }));

    #[cfg(feature = "debug")]
    app.add_plugin(WorldInspectorPlugin::new());
    // Board plugin options
    app.insert_resource(BoardOptions {
        bomb_count: 40,
        map_size: (20, 20),
        safe_start: false,
        tile_padding: 3.0,
        ..Default::default()
    });
    app.add_plugin(BoardPlugin);

    app.add_startup_system(camera_setup);

    app.run();

    println!("Hello, world!");
}
