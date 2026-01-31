-- Camera configuration and logic
-- Hot-reload this file to change camera behavior

Camera = {
    follow_speed = 5.0,
    -- Deadzone: camera won't move if target is within this distance
    deadzone = 0
}

-- Updates camera to follow target
-- Called every frame when in game
function update_camera(target_id)
    local dt = get_delta_time()

    -- Get target position
    local tx, ty = get_position(target_id)

    -- Get current camera position
    local cx, cy = get_camera_position()

    -- Calculate distance
    local dx = tx - cx
    local dy = ty - cy
    local dist = math.sqrt(dx * dx + dy * dy)

    -- Apply deadzone
    if dist <= Camera.deadzone then
        return
    end

    -- Lerp camera toward target
    local lerp_factor = math.min(Camera.follow_speed * dt, 1.0)
    local new_x = cx + dx * lerp_factor
    local new_y = cy + dy * lerp_factor

    set_camera_position(new_x, new_y)
end
