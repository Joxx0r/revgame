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

/// Health component for entities that can take damage
#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
        }
    }
}
