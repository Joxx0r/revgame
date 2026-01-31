//! RevGame - Bevy game client for RevBackend
//!
//! This crate provides a game client that integrates with the RevBackend
//! game server for authentication, session management, and matchmaking.

pub mod api;

#[cfg(feature = "graphics")]
pub mod game;
#[cfg(feature = "graphics")]
pub mod plugins;

pub use api::ApiClient;

#[cfg(feature = "graphics")]
pub use game::{ConnectionStatus, CurrentPlayer, GameState};
#[cfg(feature = "graphics")]
pub use plugins::{ApiClientResource, ApiPlugin};
