use bevy::prelude::*;

use super::state::{ConnectionStatus, CurrentPlayer, GameState};

/// System to handle loading completion
pub fn check_loading_complete(
    connection_status: Res<ConnectionStatus>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Once we're connected (or failed), move to main menu
    if *connection_status != ConnectionStatus::Connecting {
        next_state.set(GameState::MainMenu);
    }
}

/// System to log state transitions
pub fn log_state_transitions(state: Res<State<GameState>>) {
    info!("Game state: {:?}", state.get());
}

/// System to display connection status
pub fn display_connection_status(connection_status: Res<ConnectionStatus>) {
    if connection_status.is_changed() {
        info!("Connection status: {:?}", *connection_status);
    }
}

/// System to display current player info
pub fn display_player_info(player: Res<CurrentPlayer>) {
    if player.is_changed() && player.is_logged_in() {
        info!(
            "Logged in as: {}",
            player.username.as_deref().unwrap_or("Unknown")
        );
    }
}
