use bevy::prelude::*;

#[cfg(feature = "debug")]
use bevy_inspector_egui::WorldInspectorPlugin;


fn camera_setup(mut commands: Commands) {
    // 2D orthographic camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn main() {
    let mut app = App::new();

    // Window setup
    app.insert_resource(WindowDescriptor {
            title: "Minesweeper".to_string(),
            width: 700.,
            height: 800.,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins);

    #[cfg(feature = "debug")]
    app.add_plugin(WorldInspectorPlugin::new());

    app.add_startup_system(camera_setup);

    app.run();

    println!("Hello, world!");
}
