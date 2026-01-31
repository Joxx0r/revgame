use bevy::prelude::*;

/// Main game state machine
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    /// Initial loading state
    #[default]
    Loading,
    /// Main menu - not authenticated
    MainMenu,
    /// Authentication in progress
    Authenticating,
    /// Lobby - authenticated, browsing sessions
    Lobby,
    /// Matchmaking - searching for a match
    Matchmaking,
    /// In game
    InGame,
}

/// Connection status to the backend
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Resource)]
pub enum ConnectionStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Error,
}

/// Current player info (when authenticated)
#[derive(Debug, Clone, Default, Resource)]
pub struct CurrentPlayer {
    pub username: Option<String>,
    pub email: Option<String>,
}

impl CurrentPlayer {
    pub fn is_logged_in(&self) -> bool {
        self.username.is_some()
    }

    pub fn clear(&mut self) {
        self.username = None;
        self.email = None;
    }
}
