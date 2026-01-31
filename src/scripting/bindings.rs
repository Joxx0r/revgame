use bevy::prelude::*;
use mlua::{Lua, Result as LuaResult};
use std::sync::{Arc, RwLock};

/// Shared game state accessible from Lua
#[derive(Clone, Resource)]
pub struct LuaGameState {
    inner: Arc<RwLock<LuaGameStateInner>>,
}

struct LuaGameStateInner {
    /// Pending sprite spawns (width, height, r, g, b, x, y, z)
    pending_spawns: Vec<PendingSpawn>,
    /// Entity positions to set
    position_updates: Vec<(u32, f32, f32)>,
    /// Entity velocities to set
    velocity_updates: Vec<(u32, f32, f32)>,
    /// Camera position to set
    camera_position: Option<(f32, f32)>,
    /// Entities to mark as player
    mark_player: Vec<u32>,
    /// Entities to mark as camera target
    mark_camera_target: Vec<u32>,
    /// Entities to mark as world element
    mark_world_element: Vec<u32>,
    /// Entity ID counter
    next_entity_id: u32,
    /// Spawned entity map (Lua ID -> Bevy Entity)
    entity_map: std::collections::HashMap<u32, Entity>,
    /// Current delta time
    delta_time: f32,
    /// Current keyboard state
    keys_pressed: std::collections::HashSet<String>,
    /// Entity positions (for reading)
    entity_positions: std::collections::HashMap<u32, (f32, f32)>,
    /// Camera position (for reading)
    current_camera_pos: (f32, f32),
    /// Health updates from Lua (entity_id, new_current_health)
    health_updates: Vec<(u32, f32)>,
    /// Entity health values synced from Bevy (lua_id -> (current, max))
    entity_health: std::collections::HashMap<u32, (f32, f32)>,
    /// Sprite size updates from Lua (entity_id, width, height)
    size_updates: Vec<(u32, f32, f32)>,
}

#[derive(Clone)]
pub struct PendingSpawn {
    pub lua_id: u32,
    pub width: f32,
    pub height: f32,
    pub color: Color,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl LuaGameState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(LuaGameStateInner {
                pending_spawns: Vec::new(),
                position_updates: Vec::new(),
                velocity_updates: Vec::new(),
                camera_position: None,
                mark_player: Vec::new(),
                mark_camera_target: Vec::new(),
                mark_world_element: Vec::new(),
                next_entity_id: 1,
                entity_map: std::collections::HashMap::new(),
                delta_time: 0.0,
                keys_pressed: std::collections::HashSet::new(),
                entity_positions: std::collections::HashMap::new(),
                current_camera_pos: (0.0, 0.0),
                health_updates: Vec::new(),
                entity_health: std::collections::HashMap::new(),
                size_updates: Vec::new(),
            })),
        }
    }

    pub fn set_delta_time(&self, dt: f32) {
        self.inner.write().unwrap().delta_time = dt;
    }

    pub fn set_key_pressed(&self, key: &str, pressed: bool) {
        let mut inner = self.inner.write().unwrap();
        if pressed {
            inner.keys_pressed.insert(key.to_uppercase());
        } else {
            inner.keys_pressed.remove(&key.to_uppercase());
        }
    }

    pub fn clear_keys(&self) {
        self.inner.write().unwrap().keys_pressed.clear();
    }

    pub fn update_entity_position(&self, lua_id: u32, x: f32, y: f32) {
        self.inner
            .write()
            .unwrap()
            .entity_positions
            .insert(lua_id, (x, y));
    }

    pub fn set_camera_position_read(&self, x: f32, y: f32) {
        self.inner.write().unwrap().current_camera_pos = (x, y);
    }

    pub fn register_entity(&self, lua_id: u32, entity: Entity) {
        self.inner.write().unwrap().entity_map.insert(lua_id, entity);
    }

    pub fn get_entity(&self, lua_id: u32) -> Option<Entity> {
        self.inner.read().unwrap().entity_map.get(&lua_id).copied()
    }

    pub fn take_pending_spawns(&self) -> Vec<PendingSpawn> {
        std::mem::take(&mut self.inner.write().unwrap().pending_spawns)
    }

    pub fn take_position_updates(&self) -> Vec<(u32, f32, f32)> {
        std::mem::take(&mut self.inner.write().unwrap().position_updates)
    }

    pub fn take_velocity_updates(&self) -> Vec<(u32, f32, f32)> {
        std::mem::take(&mut self.inner.write().unwrap().velocity_updates)
    }

    pub fn take_camera_position(&self) -> Option<(f32, f32)> {
        self.inner.write().unwrap().camera_position.take()
    }

    pub fn take_mark_player(&self) -> Vec<u32> {
        std::mem::take(&mut self.inner.write().unwrap().mark_player)
    }

    pub fn take_mark_camera_target(&self) -> Vec<u32> {
        std::mem::take(&mut self.inner.write().unwrap().mark_camera_target)
    }

    pub fn take_mark_world_element(&self) -> Vec<u32> {
        std::mem::take(&mut self.inner.write().unwrap().mark_world_element)
    }

    pub fn update_entity_health(&self, lua_id: u32, current: f32, max: f32) {
        self.inner
            .write()
            .unwrap()
            .entity_health
            .insert(lua_id, (current, max));
    }

    pub fn take_health_updates(&self) -> Vec<(u32, f32)> {
        std::mem::take(&mut self.inner.write().unwrap().health_updates)
    }

    pub fn take_size_updates(&self) -> Vec<(u32, f32, f32)> {
        std::mem::take(&mut self.inner.write().unwrap().size_updates)
    }
}

impl Default for LuaGameState {
    fn default() -> Self {
        Self::new()
    }
}

/// Setup Lua bindings for the game state
pub fn setup_lua_bindings(lua: &Lua, game_state: LuaGameState) -> LuaResult<()> {
    let globals = lua.globals();

    // Clone game_state for each closure
    let gs = game_state.clone();
    globals.set(
        "spawn_sprite",
        lua.create_function(move |_, (w, h, r, g, b, x, y, z): (f32, f32, f32, f32, f32, f32, f32, f32)| {
            let mut inner = gs.inner.write().unwrap();
            let lua_id = inner.next_entity_id;
            inner.next_entity_id += 1;
            inner.pending_spawns.push(PendingSpawn {
                lua_id,
                width: w,
                height: h,
                color: Color::srgb(r, g, b),
                x,
                y,
                z,
            });
            Ok(lua_id)
        })?,
    )?;

    let gs = game_state.clone();
    globals.set(
        "get_position",
        lua.create_function(move |_, entity_id: u32| {
            let inner = gs.inner.read().unwrap();
            if let Some((x, y)) = inner.entity_positions.get(&entity_id) {
                Ok((*x, *y))
            } else {
                Ok((0.0, 0.0))
            }
        })?,
    )?;

    let gs = game_state.clone();
    globals.set(
        "set_position",
        lua.create_function(move |_, (entity_id, x, y): (u32, f32, f32)| {
            gs.inner.write().unwrap().position_updates.push((entity_id, x, y));
            Ok(())
        })?,
    )?;

    let gs = game_state.clone();
    globals.set(
        "set_velocity",
        lua.create_function(move |_, (entity_id, vx, vy): (u32, f32, f32)| {
            gs.inner.write().unwrap().velocity_updates.push((entity_id, vx, vy));
            Ok(())
        })?,
    )?;

    let gs = game_state.clone();
    globals.set(
        "is_key_pressed",
        lua.create_function(move |_, key: String| {
            let inner = gs.inner.read().unwrap();
            Ok(inner.keys_pressed.contains(&key.to_uppercase()))
        })?,
    )?;

    let gs = game_state.clone();
    globals.set(
        "get_delta_time",
        lua.create_function(move |_, ()| {
            let inner = gs.inner.read().unwrap();
            Ok(inner.delta_time)
        })?,
    )?;

    let gs = game_state.clone();
    globals.set(
        "get_camera_position",
        lua.create_function(move |_, ()| {
            let inner = gs.inner.read().unwrap();
            Ok(inner.current_camera_pos)
        })?,
    )?;

    let gs = game_state.clone();
    globals.set(
        "set_camera_position",
        lua.create_function(move |_, (x, y): (f32, f32)| {
            gs.inner.write().unwrap().camera_position = Some((x, y));
            Ok(())
        })?,
    )?;

    let gs = game_state.clone();
    globals.set(
        "mark_as_player",
        lua.create_function(move |_, entity_id: u32| {
            gs.inner.write().unwrap().mark_player.push(entity_id);
            Ok(())
        })?,
    )?;

    let gs = game_state.clone();
    globals.set(
        "mark_as_camera_target",
        lua.create_function(move |_, entity_id: u32| {
            gs.inner.write().unwrap().mark_camera_target.push(entity_id);
            Ok(())
        })?,
    )?;

    let gs = game_state.clone();
    globals.set(
        "mark_as_world_element",
        lua.create_function(move |_, entity_id: u32| {
            gs.inner.write().unwrap().mark_world_element.push(entity_id);
            Ok(())
        })?,
    )?;

    let gs = game_state.clone();
    globals.set(
        "set_health",
        lua.create_function(move |_, (entity_id, health): (u32, f32)| {
            gs.inner.write().unwrap().health_updates.push((entity_id, health));
            Ok(())
        })?,
    )?;

    let gs = game_state.clone();
    globals.set(
        "get_health",
        lua.create_function(move |_, entity_id: u32| {
            let inner = gs.inner.read().unwrap();
            if let Some((current, max)) = inner.entity_health.get(&entity_id) {
                Ok((*current, *max))
            } else {
                Ok((0.0, 0.0))
            }
        })?,
    )?;

    let gs = game_state.clone();
    globals.set(
        "set_sprite_size",
        lua.create_function(move |_, (entity_id, w, h): (u32, f32, f32)| {
            gs.inner.write().unwrap().size_updates.push((entity_id, w, h));
            Ok(())
        })?,
    )?;

    globals.set(
        "log",
        lua.create_function(move |_, msg: String| {
            info!("[Lua] {}", msg);
            Ok(())
        })?,
    )?;

    Ok(())
}
