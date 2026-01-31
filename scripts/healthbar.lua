-- Healthbar configuration and logic
-- Hot-reload this file to change appearance without restarting

Healthbar = {
    width = 50,
    height = 6,
    offset_y = 35,
    bg_color = { r = 0.2, g = 0.2, b = 0.2 },
    fg_color = { r = 0.2, g = 0.8, b = 0.2 },
    bg_id = nil,
    fg_id = nil,
    spawned = false
}

-- Spawn the healthbar sprites (called once after player spawns)
function spawn_healthbar(player_id)
    local px, py = get_position(player_id)

    -- Background bar (full width, dark gray)
    Healthbar.bg_id = spawn_sprite(
        Healthbar.width, Healthbar.height,
        Healthbar.bg_color.r, Healthbar.bg_color.g, Healthbar.bg_color.b,
        px, py + Healthbar.offset_y, 2
    )

    -- Foreground bar (resized based on health ratio)
    Healthbar.fg_id = spawn_sprite(
        Healthbar.width, Healthbar.height,
        Healthbar.fg_color.r, Healthbar.fg_color.g, Healthbar.fg_color.b,
        px, py + Healthbar.offset_y, 3
    )

    Healthbar.spawned = true
    log("Healthbar spawned")
end

-- Update healthbar position and width every frame
function update_healthbar(player_id)
    if not Healthbar.spawned then
        spawn_healthbar(player_id)
        return
    end

    local px, py = get_position(player_id)

    -- Get current health
    local current, max = get_health(player_id)
    if max <= 0 then max = 1 end
    local ratio = current / max
    if ratio < 0 then ratio = 0 end
    if ratio > 1 then ratio = 1 end

    -- Position background bar above player
    local bar_y = py + Healthbar.offset_y
    set_position(Healthbar.bg_id, px, bar_y)

    -- Foreground bar: shrink width and shift left to keep left-aligned
    local fg_width = Healthbar.width * ratio
    local offset_x = (Healthbar.width - fg_width) / 2
    set_position(Healthbar.fg_id, px - offset_x, bar_y)
    set_sprite_size(Healthbar.fg_id, fg_width, Healthbar.height)
end
