use bevy::prelude::*;

use super::components::{CameraTarget, MoveSpeed, Player, Velocity};

/// Spawns the player entity
pub fn spawn_player(mut commands: Commands) {
    info!("Spawning player...");

    let player_color = Color::srgb(0.204, 0.596, 0.859); // Blue #3498db
    let player_size = Vec2::new(50.0, 50.0);

    commands.spawn((
        Sprite {
            color: player_color,
            custom_size: Some(player_size),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        Player,
        Velocity::default(),
        MoveSpeed::default(),
        CameraTarget,
    ));

    info!("Player spawned at origin");
}

/// Despawns the player (for cleanup when leaving InGame state)
pub fn despawn_player(mut commands: Commands, query: Query<Entity, With<Player>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    info!("Player despawned");
}

/// Reads keyboard input and updates player velocity
pub fn player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &MoveSpeed), With<Player>>,
) {
    for (mut velocity, speed) in query.iter_mut() {
        let mut direction = Vec2::ZERO;

        // WASD controls
        if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
            direction.y -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }

        // Normalize diagonal movement to prevent faster diagonal speed
        if direction != Vec2::ZERO {
            direction = direction.normalize();
        }

        velocity.x = direction.x * speed.0;
        velocity.y = direction.y * speed.0;
    }
}

/// Applies velocity to player transform
pub fn player_movement(time: Res<Time>, mut query: Query<(&Velocity, &mut Transform), With<Player>>) {
    let delta = time.delta_secs();

    for (velocity, mut transform) in query.iter_mut() {
        transform.translation.x += velocity.x * delta;
        transform.translation.y += velocity.y * delta;
    }
}
