use bevy::prelude::*;

/// Main game state machine
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    /// Initial loading state
    #[default]
    Loading,
    /// Main menu
    MainMenu,
    /// In game
    InGame,
}
