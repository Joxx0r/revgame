use bevy::prelude::*;
use bevy::render::{
    settings::{Backends, RenderCreation, WgpuSettings},
    RenderPlugin,
};
use revgame::{game, GameState};

#[cfg(feature = "scripting")]
use revgame::scripting::check_script_changes;

fn main() {
    let mut app = App::new();

    app.add_plugins(
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
    // Setup systems
    .add_systems(OnEnter(GameState::Loading), setup);

    // Use Lua scripting if enabled, otherwise use Rust systems
    #[cfg(feature = "scripting")]
    {
        app.add_systems(Startup, game::init_lua_scripting)
            .add_systems(
                OnEnter(GameState::InGame),
                (game::lua_spawn_world, game::lua_spawn_player),
            )
            .add_systems(
                Update,
                (
                    check_script_changes,
                    game::lua_update_time,
                    game::lua_update_input,
                    game::lua_sync_positions,
                    game::lua_update_player,
                    game::lua_update_healthbar,
                    game::lua_update_camera,
                    game::lua_process_commands,
                )
                    .chain()
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                OnExit(GameState::InGame),
                (game::despawn_world, game::despawn_player),
            );
    }

    #[cfg(not(feature = "scripting"))]
    {
        app.add_systems(
            OnEnter(GameState::InGame),
            (game::spawn_world, game::spawn_player, game::spawn_agent),
        )
        .add_systems(
            Update,
            (
                game::player_input,
                game::stamina_system,
                game::player_movement,
                game::agent_behavior,
                game::camera_follow,
            )
                .chain()
                .run_if(in_state(GameState::InGame)),
        )
        .add_systems(
            OnExit(GameState::InGame),
            (game::despawn_world, game::despawn_player, game::despawn_agents),
        );
    }

    app.run();
}

fn setup(mut commands: Commands, mut next_state: ResMut<NextState<GameState>>) {
    // Spawn a 2D camera
    commands.spawn(Camera2d);
    info!("RevGame started");
    // Go straight to game
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
