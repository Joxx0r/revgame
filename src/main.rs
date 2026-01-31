use bevy::prelude::*;
use revgame::{game, plugins::ApiPlugin, GameState};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "RevGame".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        // Initialize game state
        .init_state::<GameState>()
        // Add API plugin
        .add_plugins(ApiPlugin::default())
        // Add game systems
        .add_systems(OnEnter(GameState::Loading), setup)
        .add_systems(
            Update,
            (
                game::display_connection_status,
                game::display_player_info,
            ),
        )
        .add_systems(OnEnter(GameState::MainMenu), on_main_menu)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn a 2D camera
    commands.spawn(Camera2d);
    info!("RevGame started - Loading...");
}

fn on_main_menu() {
    info!("Entered Main Menu - Ready for authentication");
    info!("Use the API client to register or login");
}
