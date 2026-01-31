use bevy::prelude::*;
use std::path::PathBuf;

use crate::game::{CameraTarget, MoveSpeed, Player, Velocity, WorldElement};
use crate::scripting::{init_script_watcher, setup_lua_bindings, LuaGameState, LuaRuntime};

/// Resource to track the player entity spawned by Lua
#[derive(Resource, Default)]
pub struct LuaPlayerEntity(pub Option<(u32, Entity)>);

/// Initialize the Lua scripting system
pub fn init_lua_scripting(mut commands: Commands) {
    // Create Lua runtime
    let mut runtime = match LuaRuntime::new() {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to create Lua runtime: {}", e);
            return;
        }
    };

    // Create game state
    let game_state = LuaGameState::new();

    // Setup bindings
    {
        let lua = runtime.lua();
        if let Err(e) = setup_lua_bindings(&lua, game_state.clone()) {
            error!("Failed to setup Lua bindings: {}", e);
            return;
        }
    }

    // Load initial scripts
    let scripts_dir = PathBuf::from("scripts");
    for script in &["world", "player", "camera"] {
        let path = scripts_dir.join(format!("{}.lua", script));
        if path.exists() {
            if let Err(e) = runtime.load_script(script, &path) {
                error!("Failed to load {}.lua: {}", script, e);
            }
        } else {
            warn!("Script not found: {:?}", path);
        }
    }

    // Initialize file watcher for hot reload
    if let Some(watcher) = init_script_watcher(scripts_dir) {
        commands.insert_resource(watcher);
    }

    commands.insert_resource(runtime);
    commands.insert_resource(game_state);
    commands.insert_resource(LuaPlayerEntity::default());

    info!("Lua scripting initialized");
}

/// Spawn world using Lua
pub fn lua_spawn_world(runtime: Option<Res<LuaRuntime>>) {
    let Some(runtime) = runtime else { return };

    if let Err(e) = (*runtime).call_function("spawn_world") {
        error!("Failed to call spawn_world: {}", e);
    }
}

/// Spawn player using Lua
pub fn lua_spawn_player(
    runtime: Option<Res<LuaRuntime>>,
    mut player_entity: Option<ResMut<LuaPlayerEntity>>,
) {
    let Some(runtime) = runtime else { return };
    let Some(ref mut player_entity) = player_entity else {
        return;
    };

    match (*runtime).call_spawn_function("spawn_player") {
        Ok(lua_id) => {
            info!("Lua spawn_player returned ID: {}", lua_id);
            // Entity will be created in process_lua_commands
            player_entity.0 = Some((lua_id, Entity::PLACEHOLDER));
        }
        Err(e) => error!("Failed to call spawn_player: {}", e),
    }
}

/// Update keyboard state for Lua
pub fn lua_update_input(keyboard: Res<ButtonInput<KeyCode>>, game_state: Option<Res<LuaGameState>>) {
    let Some(game_state) = game_state else { return };

    // Clear and update key states
    game_state.clear_keys();

    let key_mappings = [
        (KeyCode::KeyW, "W"),
        (KeyCode::KeyA, "A"),
        (KeyCode::KeyS, "S"),
        (KeyCode::KeyD, "D"),
        (KeyCode::ArrowUp, "UP"),
        (KeyCode::ArrowDown, "DOWN"),
        (KeyCode::ArrowLeft, "LEFT"),
        (KeyCode::ArrowRight, "RIGHT"),
    ];

    for (code, name) in key_mappings {
        if keyboard.pressed(code) {
            game_state.set_key_pressed(name, true);
        }
    }
}

/// Update delta time for Lua
pub fn lua_update_time(time: Res<Time>, game_state: Option<Res<LuaGameState>>) {
    let Some(game_state) = game_state else { return };
    game_state.set_delta_time(time.delta_secs());
}

/// Sync entity positions from Bevy to Lua (for reading)
pub fn lua_sync_positions(
    game_state: Option<Res<LuaGameState>>,
    player_entity: Option<Res<LuaPlayerEntity>>,
    transforms: Query<&Transform>,
    camera_query: Query<&Transform, With<Camera2d>>,
) {
    let Some(game_state) = game_state else { return };
    let Some(player_entity) = player_entity else {
        return;
    };

    // Sync player position
    if let Some((lua_id, entity)) = player_entity.0 {
        if entity != Entity::PLACEHOLDER {
            if let Ok(transform) = transforms.get(entity) {
                game_state.update_entity_position(lua_id, transform.translation.x, transform.translation.y);
            }
        }
    }

    // Sync camera position
    if let Ok(camera_transform) = camera_query.get_single() {
        game_state.set_camera_position_read(
            camera_transform.translation.x,
            camera_transform.translation.y,
        );
    }
}

/// Call Lua update functions
pub fn lua_update_player(
    runtime: Option<Res<LuaRuntime>>,
    player_entity: Option<Res<LuaPlayerEntity>>,
) {
    let Some(runtime) = runtime else { return };
    let Some(player_entity) = player_entity else {
        return;
    };

    if let Some((lua_id, entity)) = player_entity.0 {
        if entity != Entity::PLACEHOLDER {
            let _ = (*runtime).call_update_function("update_player", lua_id, 0.0);
        }
    }
}

/// Call Lua camera update
pub fn lua_update_camera(
    runtime: Option<Res<LuaRuntime>>,
    player_entity: Option<Res<LuaPlayerEntity>>,
) {
    let Some(runtime) = runtime else { return };
    let Some(player_entity) = player_entity else {
        return;
    };

    if let Some((lua_id, entity)) = player_entity.0 {
        if entity != Entity::PLACEHOLDER {
            let _ = (*runtime).call_update_function("update_camera", lua_id, 0.0);
        }
    }
}

/// Process commands from Lua (spawn entities, update positions, etc.)
pub fn lua_process_commands(
    mut commands: Commands,
    game_state: Option<Res<LuaGameState>>,
    mut player_entity: Option<ResMut<LuaPlayerEntity>>,
    mut transforms: Query<&mut Transform, Without<Camera2d>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let Some(game_state) = game_state else { return };

    // Process pending spawns
    for spawn in game_state.take_pending_spawns() {
        let entity = commands
            .spawn((
                Sprite {
                    color: spawn.color,
                    custom_size: Some(Vec2::new(spawn.width, spawn.height)),
                    ..default()
                },
                Transform::from_xyz(spawn.x, spawn.y, spawn.z),
            ))
            .id();

        // Register entity mapping
        game_state.register_entity(spawn.lua_id, entity);

        // Update player entity if this was the player spawn
        if let Some(ref mut player_entity) = player_entity {
            if let Some((lua_id, _)) = player_entity.0 {
                if lua_id == spawn.lua_id {
                    player_entity.0 = Some((lua_id, entity));
                }
            }
        }
    }

    // Process marker components
    for lua_id in game_state.take_mark_player() {
        if let Some(entity) = game_state.get_entity(lua_id) {
            commands.entity(entity).insert((
                Player,
                Velocity::default(),
                MoveSpeed::default(),
            ));
        }
    }

    for lua_id in game_state.take_mark_camera_target() {
        if let Some(entity) = game_state.get_entity(lua_id) {
            commands.entity(entity).insert(CameraTarget);
        }
    }

    for lua_id in game_state.take_mark_world_element() {
        if let Some(entity) = game_state.get_entity(lua_id) {
            commands.entity(entity).insert(WorldElement);
        }
    }

    // Process position updates
    for (lua_id, x, y) in game_state.take_position_updates() {
        if let Some(entity) = game_state.get_entity(lua_id) {
            if let Ok(mut transform) = transforms.get_mut(entity) {
                transform.translation.x = x;
                transform.translation.y = y;
            }
        }
    }

    // Process camera position
    if let Some((x, y)) = game_state.take_camera_position() {
        if let Ok(mut camera_transform) = camera_query.get_single_mut() {
            camera_transform.translation.x = x;
            camera_transform.translation.y = y;
        }
    }
}
