use bevy::prelude::*;
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind, notify::RecursiveMode};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::sync::Mutex;
use std::time::Duration;

use super::LuaRuntime;

/// Resource that watches the scripts directory for changes
#[derive(Resource)]
pub struct ScriptWatcher {
    rx: Mutex<Receiver<Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify_debouncer_mini::notify::Error>>>,
    scripts_dir: PathBuf,
    // Keep the debouncer alive - wrapped in Box to make it Send
    _debouncer: Box<notify_debouncer_mini::Debouncer<notify_debouncer_mini::notify::RecommendedWatcher>>,
}

// Safety: ScriptWatcher is only accessed from the main thread via Bevy systems
unsafe impl Send for ScriptWatcher {}
unsafe impl Sync for ScriptWatcher {}

impl ScriptWatcher {
    pub fn new(scripts_dir: PathBuf) -> Result<Self, notify_debouncer_mini::notify::Error> {
        let (tx, rx) = channel();

        let mut debouncer = new_debouncer(Duration::from_millis(200), tx)?;
        debouncer.watcher().watch(&scripts_dir, RecursiveMode::Recursive)?;

        info!("Script watcher initialized for: {:?}", scripts_dir);

        Ok(Self {
            rx: Mutex::new(rx),
            scripts_dir,
            _debouncer: Box::new(debouncer),
        })
    }

    pub fn scripts_dir(&self) -> &PathBuf {
        &self.scripts_dir
    }

    /// Try to receive pending events
    pub fn try_recv(&self) -> Vec<notify_debouncer_mini::DebouncedEvent> {
        let rx = self.rx.lock().unwrap();
        let mut events = Vec::new();
        while let Ok(result) = rx.try_recv() {
            if let Ok(mut batch) = result {
                events.append(&mut batch);
            }
        }
        events
    }
}

/// System that checks for script changes and triggers reloads
pub fn check_script_changes(
    watcher: Option<Res<ScriptWatcher>>,
    mut runtime: Option<ResMut<LuaRuntime>>,
) {
    let Some(watcher) = watcher else { return };
    let Some(ref mut runtime) = runtime else { return };

    let events = watcher.try_recv();
    for event in events {
        if event.kind == DebouncedEventKind::Any {
            let path = &event.path;

            // Only reload .lua files
            if path.extension().map(|e| e == "lua").unwrap_or(false) {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    match runtime.reload_script(name, path) {
                        Ok(true) => info!("Hot-reloaded: {}", name),
                        Ok(false) => {} // No change
                        Err(e) => error!("Failed to reload {}: {}", name, e),
                    }
                }
            }
        }
    }
}

/// Initialize the script watcher for the scripts directory
pub fn init_script_watcher(scripts_dir: PathBuf) -> Option<ScriptWatcher> {
    match ScriptWatcher::new(scripts_dir) {
        Ok(watcher) => Some(watcher),
        Err(e) => {
            error!("Failed to initialize script watcher: {}", e);
            None
        }
    }
}
