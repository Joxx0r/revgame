use bevy::prelude::*;

use super::components::WorldElement;

/// Spawns the game world: ground and grid markers for visual reference
pub fn spawn_world(mut commands: Commands) {
    info!("Spawning game world...");

    // Ground - large dark green rectangle
    let ground_color = Color::srgb(0.176, 0.353, 0.153); // Dark green #2d5a27
    let ground_size = Vec2::new(2000.0, 2000.0);

    commands.spawn((
        Sprite {
            color: ground_color,
            custom_size: Some(ground_size),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -1.0), // Behind everything
        WorldElement,
    ));

    // Grid markers - small gray squares every 200 pixels
    let marker_color = Color::srgb(0.333, 0.333, 0.333); // Gray #555555
    let marker_size = Vec2::new(20.0, 20.0);
    let grid_spacing = 200.0;
    let grid_range = 5; // -5 to 5 = 11x11 grid

    for x in -grid_range..=grid_range {
        for y in -grid_range..=grid_range {
            // Skip center (player spawn point)
            if x == 0 && y == 0 {
                continue;
            }

            let pos_x = x as f32 * grid_spacing;
            let pos_y = y as f32 * grid_spacing;

            commands.spawn((
                Sprite {
                    color: marker_color,
                    custom_size: Some(marker_size),
                    ..default()
                },
                Transform::from_xyz(pos_x, pos_y, -0.5), // Above ground, below player
                WorldElement,
            ));
        }
    }

    let marker_count = (grid_range * 2 + 1) * (grid_range * 2 + 1) - 1;
    info!("World spawned with {} grid markers", marker_count);
}

/// Despawns all world elements (for cleanup when leaving InGame state)
pub fn despawn_world(mut commands: Commands, query: Query<Entity, With<WorldElement>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    info!("World despawned");
}
