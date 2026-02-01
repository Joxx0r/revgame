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

/// Stamina component - drains while moving, recharges when stopped.
/// Low stamina reduces movement speed proportionally.
#[derive(Component)]
pub struct Stamina {
    pub current: f32,
    pub max: f32,
    /// Stamina units drained per second while moving
    pub drain_rate: f32,
    /// Stamina units recharged per second while stopped
    pub recharge_rate: f32,
}

impl Default for Stamina {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
            drain_rate: 20.0,
            recharge_rate: 30.0,
        }
    }
}

/// State machine for the orbiter AI agent
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AgentState {
    /// Orbiting around the player at a fixed radius
    Circling,
    /// Moving toward the player to interact
    Approaching,
    /// At the player, performing an interaction (bump)
    Interacting,
    /// Returning from player back to orbit radius
    Returning,
}

/// AI agent that orbits around the player
#[derive(Component)]
pub struct OrbiterAgent {
    /// Current behavior state
    pub state: AgentState,
    /// Orbit radius in pixels
    pub orbit_radius: f32,
    /// Angular speed in radians per second
    pub orbit_speed: f32,
    /// Current angle on the orbit circle (radians)
    pub angle: f32,
    /// Movement speed when approaching/returning (pixels per second)
    pub move_speed: f32,
    /// Timer for interaction duration
    pub interact_timer: f32,
    /// Timer for how long to circle before next approach
    pub circle_timer: f32,
    /// Duration of interaction pause
    pub interact_duration: f32,
    /// Duration of circling before approaching
    pub circle_duration: f32,
}
