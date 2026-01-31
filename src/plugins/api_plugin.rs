use bevy::prelude::*;

use crate::api::ApiClient;
use crate::game::{ConnectionStatus, CurrentPlayer, GameState};

/// Plugin for RevBackend API integration
pub struct ApiPlugin {
    pub base_url: String,
}

impl Default for ApiPlugin {
    fn default() -> Self {
        Self {
            base_url: std::env::var("REVBACKEND_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
        }
    }
}

impl Plugin for ApiPlugin {
    fn build(&self, app: &mut App) {
        // Insert API client as a resource
        let client = ApiClient::new(&self.base_url);

        app.insert_resource(ApiClientResource(client))
            .insert_resource(ConnectionStatus::default())
            .insert_resource(CurrentPlayer::default())
            .add_systems(Startup, setup_api_client)
            .add_systems(
                Update,
                check_connection.run_if(in_state(GameState::Loading)),
            );

        info!("ApiPlugin initialized with base URL: {}", self.base_url);
    }
}

/// Resource wrapper for the API client
#[derive(Resource)]
pub struct ApiClientResource(pub ApiClient);

impl std::ops::Deref for ApiClientResource {
    type Target = ApiClient;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// System to set up the API client
fn setup_api_client(mut connection_status: ResMut<ConnectionStatus>) {
    info!("Setting up API client...");
    *connection_status = ConnectionStatus::Connecting;
}

/// System to check the connection status
fn check_connection(
    mut connection_status: ResMut<ConnectionStatus>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // For now, just mark as connected and move to main menu
    // In a real implementation, this would ping the health endpoint
    if *connection_status == ConnectionStatus::Connecting {
        *connection_status = ConnectionStatus::Connected;
        next_state.set(GameState::MainMenu);
        info!("API connection established");
    }
}
