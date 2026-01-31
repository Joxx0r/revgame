use bevy::prelude::*;
use bevy::render::{
    settings::{Backends, RenderCreation, WgpuSettings},
    RenderPlugin,
};
use revgame::{game, plugins::ApiPlugin, GameState};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "RevGame".to_string(),
                        resolution: (1280.0, 720.0).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        backends: Some(platform_backends()),
                        ..default()
                    }),
                    ..default()
                }),
        )
        // Initialize game state
        .init_state::<GameState>()
        // Add API plugin
        .add_plugins(ApiPlugin::default())
        // Setup systems
        .add_systems(OnEnter(GameState::Loading), setup)
        .add_systems(
            Update,
            (
                game::display_connection_status,
                game::display_player_info,
            ),
        )
        .add_systems(OnEnter(GameState::MainMenu), on_main_menu)
        // InGame systems
        .add_systems(
            OnEnter(GameState::InGame),
            (game::spawn_world, game::spawn_player),
        )
        .add_systems(
            Update,
            (
                game::player_input,
                game::player_movement,
                game::camera_follow,
            )
                .chain()
                .run_if(in_state(GameState::InGame)),
        )
        .add_systems(
            OnExit(GameState::InGame),
            (game::despawn_world, game::despawn_player),
        )
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn a 2D camera
    commands.spawn(Camera2d);
    info!("RevGame started - Loading...");
}

fn on_main_menu(mut next_state: ResMut<NextState<GameState>>) {
    info!("Entered Main Menu - Starting game...");
    // For testing: immediately transition to InGame
    next_state.set(GameState::InGame);
}

/// Returns the appropriate graphics backend for the current platform
fn platform_backends() -> Backends {
    #[cfg(target_os = "linux")]
    {
        // Use Vulkan on Linux (works with software renderer like llvmpipe)
        Backends::VULKAN
    }

    #[cfg(target_os = "windows")]
    {
        // Use DX12 on Windows for best performance
        Backends::DX12
    }

    #[cfg(target_os = "macos")]
    {
        // Use Metal on macOS
        Backends::METAL
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        // Fallback to primary backend for other platforms
        Backends::PRIMARY
    }
}
