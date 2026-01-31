use bevy::prelude::*;
use mlua::{Lua, Result as LuaResult};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

/// Resource that manages the Lua runtime
#[derive(Resource)]
pub struct LuaRuntime {
    lua: Arc<RwLock<Lua>>,
    loaded_scripts: HashMap<String, String>,
}

impl LuaRuntime {
    pub fn new() -> LuaResult<Self> {
        let lua = Lua::new();

        Ok(Self {
            lua: Arc::new(RwLock::new(lua)),
            loaded_scripts: HashMap::new(),
        })
    }

    /// Load a script from a file path
    pub fn load_script(&mut self, name: &str, path: &Path) -> LuaResult<()> {
        let script = std::fs::read_to_string(path)?;
        self.load_script_content(name, &script)
    }

    /// Load a script from string content
    pub fn load_script_content(&mut self, name: &str, content: &str) -> LuaResult<()> {
        let lua = self.lua.write().unwrap();
        lua.load(content).set_name(name).exec()?;
        drop(lua);
        self.loaded_scripts.insert(name.to_string(), content.to_string());
        info!("Loaded Lua script: {}", name);
        Ok(())
    }

    /// Reload a script (re-execute its content)
    pub fn reload_script(&mut self, name: &str, path: &Path) -> LuaResult<bool> {
        let new_content = std::fs::read_to_string(path)?;

        // Check if content actually changed
        if let Some(old_content) = self.loaded_scripts.get(name) {
            if old_content == &new_content {
                return Ok(false);
            }
        }

        self.load_script_content(name, &new_content)?;
        info!("Hot-reloaded Lua script: {}", name);
        Ok(true)
    }

    /// Call a Lua function with no arguments
    pub fn call_function(&self, name: &str) -> LuaResult<()> {
        let lua = self.lua.read().unwrap();
        let func: mlua::Function = lua.globals().get(name)?;
        func.call::<()>(())?;
        Ok(())
    }

    /// Call a Lua function that returns an entity ID
    pub fn call_spawn_function(&self, name: &str) -> LuaResult<u32> {
        let lua = self.lua.read().unwrap();
        let func: mlua::Function = lua.globals().get(name)?;
        let result: u32 = func.call(())?;
        Ok(result)
    }

    /// Call a Lua update function with entity ID and delta time
    pub fn call_update_function(&self, name: &str, entity_id: u32, delta: f32) -> LuaResult<()> {
        let lua = self.lua.read().unwrap();
        let func: mlua::Function = lua.globals().get(name)?;
        func.call::<()>((entity_id, delta))?;
        Ok(())
    }

    /// Get the Lua instance for binding setup
    pub fn lua(&self) -> std::sync::RwLockReadGuard<'_, Lua> {
        self.lua.read().unwrap()
    }

    /// Get mutable Lua instance for binding setup
    pub fn lua_mut(&self) -> std::sync::RwLockWriteGuard<'_, Lua> {
        self.lua.write().unwrap()
    }
}

impl Default for LuaRuntime {
    fn default() -> Self {
        Self::new().expect("Failed to create Lua runtime")
    }
}
