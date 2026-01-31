use bevy::prelude::*;

use super::components::CameraTarget;

/// Smoothly moves the camera to follow the target entity
pub fn camera_follow(
    time: Res<Time>,
    target_query: Query<&Transform, (With<CameraTarget>, Without<Camera2d>)>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    // Get target position
    let Ok(target_transform) = target_query.get_single() else {
        return; // No target to follow
    };

    // Get camera
    let Ok(mut camera_transform) = camera_query.get_single_mut() else {
        return; // No camera
    };

    let target_pos = target_transform.translation;
    let camera_pos = camera_transform.translation;

    // Smooth follow using lerp
    // Higher values = faster follow (1.0 = instant, 0.1 = slow)
    let follow_speed = 5.0;
    let lerp_factor = (follow_speed * time.delta_secs()).min(1.0);

    // Only lerp X and Y, keep camera Z unchanged
    // Use Bevy's FloatExt::lerp
    camera_transform.translation.x = FloatExt::lerp(camera_pos.x, target_pos.x, lerp_factor);
    camera_transform.translation.y = FloatExt::lerp(camera_pos.y, target_pos.y, lerp_factor);
}
