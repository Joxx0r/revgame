-- Player configuration and logic
-- Hot-reload this file to change behavior without restarting

Player = {
    speed = 200,
    size = 50,
    color = { r = 0.204, g = 0.596, b = 0.859 },  -- #3498db blue
    max_health = 100
}

-- Spawns the player entity
-- Returns the Lua entity ID
function spawn_player()
    log("Spawning player...")
    local id = spawn_sprite(
        Player.size, Player.size,
        Player.color.r, Player.color.g, Player.color.b,
        0, 0, 0
    )
    mark_as_player(id)
    mark_as_camera_target(id)
    set_health(id, Player.max_health)
    log("Player spawned with ID: " .. tostring(id))
    return id
end

-- Updates player based on input
-- Called every frame when in game
function update_player(player_id)
    local dt = get_delta_time()
    local vx, vy = 0, 0

    -- Read input
    if is_key_pressed("W") or is_key_pressed("UP") then
        vy = 1
    end
    if is_key_pressed("S") or is_key_pressed("DOWN") then
        vy = vy - 1
    end
    if is_key_pressed("A") or is_key_pressed("LEFT") then
        vx = -1
    end
    if is_key_pressed("D") or is_key_pressed("RIGHT") then
        vx = vx + 1
    end

    -- Normalize diagonal movement
    if vx ~= 0 and vy ~= 0 then
        local len = math.sqrt(vx * vx + vy * vy)
        vx = vx / len
        vy = vy / len
    end

    -- Apply movement
    local x, y = get_position(player_id)
    local new_x = x + vx * Player.speed * dt
    local new_y = y + vy * Player.speed * dt
    set_position(player_id, new_x, new_y)
end
