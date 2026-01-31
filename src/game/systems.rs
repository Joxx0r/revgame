use bevy::prelude::*;

use super::state::GameState;

/// System to log state transitions
pub fn log_state_transitions(state: Res<State<GameState>>) {
    info!("Game state: {:?}", state.get());
}
