-- World configuration and spawning
-- Hot-reload this file to change world layout

World = {
    ground_size = 2000,
    ground_color = { r = 0.176, g = 0.353, b = 0.153 },  -- #2d5a27 dark green
    grid_spacing = 200,
    grid_size = 20,
    grid_color = { r = 0.333, g = 0.333, b = 0.333 }  -- #555555 gray
}

-- Store spawned world element IDs for cleanup
world_elements = {}

-- Spawns the world (ground and grid markers)
function spawn_world()
    log("Spawning world...")
    world_elements = {}

    -- Spawn ground
    local ground_id = spawn_sprite(
        World.ground_size, World.ground_size,
        World.ground_color.r, World.ground_color.g, World.ground_color.b,
        0, 0, -1
    )
    mark_as_world_element(ground_id)
    table.insert(world_elements, ground_id)

    -- Spawn grid markers
    local half = World.ground_size / 2
    local count = 0
    for x = -half, half, World.grid_spacing do
        for y = -half, half, World.grid_spacing do
            -- Skip center (where player spawns)
            if not (x == 0 and y == 0) then
                local marker_id = spawn_sprite(
                    World.grid_size, World.grid_size,
                    World.grid_color.r, World.grid_color.g, World.grid_color.b,
                    x, y, -0.5
                )
                mark_as_world_element(marker_id)
                table.insert(world_elements, marker_id)
                count = count + 1
            end
        end
    end

    log("World spawned with " .. tostring(count) .. " grid markers")
end

-- Returns the list of world element IDs (for cleanup)
function get_world_elements()
    return world_elements
end
