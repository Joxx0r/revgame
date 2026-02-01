use bevy::prelude::*;

use super::components::{AgentState, OrbiterAgent, Player};

/// Spawns an orbiter agent entity
pub fn spawn_agent(mut commands: Commands) {
    info!("Spawning orbiter agent...");

    let agent_color = Color::srgb(0.906, 0.298, 0.235); // Red #e74c3c
    let agent_size = Vec2::new(30.0, 30.0);

    let orbit_radius = 150.0;
    let start_angle: f32 = 0.0;

    commands.spawn((
        Sprite {
            color: agent_color,
            custom_size: Some(agent_size),
            ..default()
        },
        Transform::from_xyz(orbit_radius, 0.0, 0.1),
        OrbiterAgent {
            state: AgentState::Circling,
            orbit_radius,
            orbit_speed: 1.5,
            angle: start_angle,
            move_speed: 300.0,
            interact_timer: 0.0,
            circle_timer: 0.0,
            interact_duration: 0.4,
            circle_duration: 5.0,
        },
    ));

    info!("Orbiter agent spawned");
}

/// Despawns all orbiter agents
pub fn despawn_agents(mut commands: Commands, query: Query<Entity, With<OrbiterAgent>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    info!("Orbiter agents despawned");
}

/// Drives the orbiter agent state machine and movement
pub fn agent_behavior(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<OrbiterAgent>)>,
    mut agent_query: Query<(&mut OrbiterAgent, &mut Transform), Without<Player>>,
) {
    let delta = time.delta_secs();

    let Ok(player_tf) = player_query.get_single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();

    for (mut agent, mut transform) in agent_query.iter_mut() {
        match agent.state {
            AgentState::Circling => {
                // Advance angle
                agent.angle += agent.orbit_speed * delta;
                if agent.angle > std::f32::consts::TAU {
                    agent.angle -= std::f32::consts::TAU;
                }

                // Position on orbit circle relative to player
                let target_x = player_pos.x + agent.orbit_radius * agent.angle.cos();
                let target_y = player_pos.y + agent.orbit_radius * agent.angle.sin();
                transform.translation.x = target_x;
                transform.translation.y = target_y;

                // Count down to next approach
                agent.circle_timer += delta;
                if agent.circle_timer >= agent.circle_duration {
                    agent.circle_timer = 0.0;
                    agent.state = AgentState::Approaching;
                }
            }

            AgentState::Approaching => {
                let agent_pos = transform.translation.truncate();
                let to_player = player_pos - agent_pos;
                let distance = to_player.length();

                if distance < 10.0 {
                    // Close enough — start interacting
                    agent.state = AgentState::Interacting;
                    agent.interact_timer = 0.0;
                } else {
                    let dir = to_player / distance;
                    transform.translation.x += dir.x * agent.move_speed * delta;
                    transform.translation.y += dir.y * agent.move_speed * delta;
                }
            }

            AgentState::Interacting => {
                // Stay near the player for a brief moment
                transform.translation.x = player_pos.x;
                transform.translation.y = player_pos.y;

                agent.interact_timer += delta;
                if agent.interact_timer >= agent.interact_duration {
                    // Compute return angle based on current offset from player
                    // (use the angle we left off at so the orbit resumes smoothly)
                    agent.state = AgentState::Returning;
                }
            }

            AgentState::Returning => {
                // Target point on the orbit circle
                let orbit_x = player_pos.x + agent.orbit_radius * agent.angle.cos();
                let orbit_y = player_pos.y + agent.orbit_radius * agent.angle.sin();
                let target = Vec2::new(orbit_x, orbit_y);

                let agent_pos = transform.translation.truncate();
                let to_orbit = target - agent_pos;
                let distance = to_orbit.length();

                if distance < 5.0 {
                    // Back on orbit — resume circling
                    transform.translation.x = orbit_x;
                    transform.translation.y = orbit_y;
                    agent.state = AgentState::Circling;
                } else {
                    // Also advance the angle while returning so the target
                    // keeps moving, creating a smooth catch-up arc
                    agent.angle += agent.orbit_speed * delta;
                    if agent.angle > std::f32::consts::TAU {
                        agent.angle -= std::f32::consts::TAU;
                    }

                    let dir = to_orbit / distance;
                    let speed = agent.move_speed * 1.2; // slightly faster to catch up
                    transform.translation.x += dir.x * speed * delta;
                    transform.translation.y += dir.y * speed * delta;
                }
            }
        }
    }
}
