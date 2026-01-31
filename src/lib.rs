//! RevGame - Bevy game client for RevBackend
//!
//! This crate provides a game client that integrates with the RevBackend
//! game server for authentication, session management, and matchmaking.

pub mod api;
pub mod game;
pub mod plugins;

pub use api::ApiClient;
pub use game::{ConnectionStatus, CurrentPlayer, GameState};
pub use plugins::{ApiClientResource, ApiPlugin};
