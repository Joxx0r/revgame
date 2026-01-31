use bevy::prelude::*;

/// Marker component for the player entity
#[derive(Component)]
pub struct Player;

/// Movement velocity component
#[derive(Component, Default)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

/// Marker for entities the camera should follow
#[derive(Component)]
pub struct CameraTarget;

/// Marker for world/environment elements
#[derive(Component)]
pub struct WorldElement;

/// Movement speed configuration
#[derive(Component)]
pub struct MoveSpeed(pub f32);

impl Default for MoveSpeed {
    fn default() -> Self {
        Self(200.0) // pixels per second
    }
}
