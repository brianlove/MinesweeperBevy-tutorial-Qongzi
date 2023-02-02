use bevy::log;
use bevy::prelude::*;
use board_plugin::BoardPlugin;
use board_plugin::resources::{BoardAssets, BoardOptions, SpriteMaterial};

#[cfg(feature = "debug")]
use bevy_inspector_egui::WorldInspectorPlugin;


#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    InGame,
    Out,
}

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
    app.add_plugin(BoardPlugin {
        running_state: AppState::InGame,
    });
    app.add_state(AppState::Out);

    app.add_startup_system(setup_board);
    app.add_startup_system(camera_setup);
    app.add_system(state_handler);

    app.run();

    println!("Hello, world!");
}

fn setup_board(
    mut commands: Commands,
    mut state: ResMut<State<AppState>>,
    asset_server: Res<AssetServer>,
) {
    // Board plugin options
    commands.insert_resource(BoardOptions {
        bomb_count: 40,
        map_size: (20, 20),
        safe_start: false,
        tile_padding: 1.,
        ..Default::default()
    });

    // Board assets
    commands.insert_resource(BoardAssets {
        label: "Default".to_string(),
        board_material: SpriteMaterial {
            color: Color::WHITE,
            ..Default::default()
        },
        tile_material: SpriteMaterial {
            color: Color::DARK_GRAY,
            ..Default::default()
        },
        covered_tile_material: SpriteMaterial {
            color: Color::GRAY,
            ..Default::default()
        },
        bomb_counter_font: asset_server.load("fonts/pixeled.ttf"),
        bomb_counter_colors: BoardAssets::default_colors(),
        flag_material: SpriteMaterial {
            texture: asset_server.load("sprites/flag.png"),
            color: Color::WHITE,
        },
        bomb_material: SpriteMaterial {
            texture: asset_server.load("sprites/bomb.png"),
            color: Color::WHITE,
        },
    });

    // Plugin activation
    state.set(AppState::InGame).unwrap();
}

fn state_handler(mut state: ResMut<State<AppState>>, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::C) {
        log::debug!("clearing detected");
        if state.current() == &AppState::InGame {
            log::info!("clearing game");
            state.set(AppState::Out).unwrap();
        }
    }
    if keys.just_pressed(KeyCode::G) {
        log::debug!("loading detected");
        if state.current() == &AppState::Out {
            log::info!("loading game");
            state.set(AppState::InGame).unwrap();
        }
    }
}
