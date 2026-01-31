//! RevGame - Bevy game client

#[cfg(feature = "graphics")]
pub mod game;
#[cfg(feature = "scripting")]
pub mod scripting;

#[cfg(feature = "graphics")]
pub use game::GameState;
