use bevy::{prelude::*, log};

#[cfg(feature="debug")]
use bevy_inspector_egui::WorldInspectorPlugin;
use board_plugin::{BoardPlugin, resources::{BoardOptions, paused::Paused, BoardAssets, SpriteMaterial}};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    InGame,
    Out,
}

#[derive(Debug, Clone, Copy)]
pub struct RestartEvent;

fn main() {
    let mut app = App::new();
    app.insert_resource(WindowDescriptor {
        title: "Mine Sweeper!".to_string(),
        width: 700.0,
        height: 800.0,
        ..Default::default()
    })
    .add_plugins(DefaultPlugins)
    .add_state(AppState::Out)
    .insert_resource(Paused(false))
    .add_plugin(BoardPlugin {
        running_state: AppState::InGame,
        out_state: AppState::Out,
    })
    .add_event::<RestartEvent>();

    #[cfg(feature="debug")]
    app.add_plugin(WorldInspectorPlugin::new());

    app.add_startup_system(camera_setup)
       .add_startup_system(setup_board)
       .add_system(state_handler)
       .add_system(restart)
       .run();
}

fn camera_setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn state_handler(
    mut state: ResMut<State<AppState>>,
    keys: Res<Input<KeyCode>>,
    mut writer: EventWriter<RestartEvent>,
    mut paused: ResMut<Paused>
) {
    if keys.just_pressed(KeyCode::Escape) {
        log::debug!("pressing `Escape` detected");
        if state.current() != &AppState::InGame { return; }
        if !paused.0 {
            log::info!("pausing game");
            paused.0 = true;
        } else {
            log::info!("resuming game");
            paused.0 = false;
        }
    }
    if keys.just_pressed(KeyCode::G) {
        log::debug!("pressing `G` detected");
        log::info!("generating new board");
        match state.current() {
            &AppState::Out => {
                state.set(AppState::InGame).unwrap();
            }
            _ => {
                state.set(AppState::Out).unwrap();
                writer.send(RestartEvent);
            }
        }
    }
}

fn restart(mut reader: EventReader<RestartEvent>, mut state: ResMut<State<AppState>>) {
    for _event in reader.iter() {
        if state.current() == &AppState::Out {
            state.set(AppState::InGame).unwrap();
            break;
        }
    }
}

fn setup_board(
    mut commands: Commands,
    mut state: ResMut<State<AppState>>,
    asset_server: Res<AssetServer>,
) {
    // Board plugin options
    commands.insert_resource(BoardOptions {
        map_size: (20, 20),
        bomb_count: 40,
        tile_padding: 3.0,
        safe_start: true,
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
    state.set(AppState::InGame).unwrap();
}