use crate::game::{GameState, SystemLabels, run_if_tick_elapsed};

mod components;
mod terrain;
mod collision;

pub use bevy::prelude::*;

use bevy::utils::HashSet;
pub use terrain::*;
pub use collision::*;
pub use components::*;

/// Plugin that adds physics components and terrain collision
pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Gravity::default());
        app.add_system_set(SystemSet::on_update(GameState::GameRunning)
            .with_run_criteria(run_if_tick_elapsed)
            .with_system(apply_gravity
                .label(SystemLabels::PhysicsGravity)
                .after(SystemLabels::Logic)
            )
            .with_system(apply_friction
                .label(SystemLabels::PhysicsFriction)
                .after(SystemLabels::Logic)
                .after(SystemLabels::PhysicsGravity)
            )
            .with_system(prepare_states
                .label(SystemLabels::PhysicsSync)
                .before(SystemLabels::PhysicsMove)
                .after(SystemLabels::Logic)
            )
            .with_system(prepare_positions
                .label(SystemLabels::PhysicsSync)
                .before(SystemLabels::PhysicsMove)
                .after(SystemLabels::Logic)
            )
            .with_system(apply_velocity
                .label(SystemLabels::PhysicsMove)
                .after(SystemLabels::Logic)
                .after(SystemLabels::PhysicsFriction)
            )
            .with_system(collide_with_terrain
                .label(SystemLabels::PhysicsCollide)
                .after(SystemLabels::PhysicsMove)
            )
        );
    }
}


const COLLISION_RETRIES: u32 = 4;
fn collide_with_terrain(
    terrain_entity: Query<&Terrain>,
    mut collidable_entities: Query<(
        &mut Position,
        &PreviousPosition,
        &CylinderShape,
        &mut Velocity,
        Option<&mut PhysicsState>
    )>,
    mut terrain_ids: Local<HashSet<TerrainId>>
) {
    const EPSILON: f32 = 0.01;
    log::debug!("(SYSTEM) collide_with_terrain");
    terrain_ids.clear();

    // Gets terrain to collide with
    let terrain: &Terrain = match terrain_entity.iter().next() {
        Some(entity) => entity,
        None => return
    };

    // For all collidable entities
    for (mut pos, prev_pos, size, mut vel, mut state) in collidable_entities.iter_mut() {
        
        let mut pos_value = pos.0;              // End point in collision
        let mut prev_pos_value = prev_pos.0;    // Start point in collision
        let initial_vel_value = vel.0;          // Initial velocity value
        let mut vel_value = vel.0;              // Velocity at start point
        let mut delta = vel_value;              // Change in motion

        // For N retries...
        for _ in 0..COLLISION_RETRIES {
            let cylinder = CylinderCollider {
                center: prev_pos_value,
                radius: size.radius,
                half_height: size.half_height
            };
            match terrain.collide_with_cylinder(&cylinder, delta, &terrain_ids) {
                Some((collision, tid)) => {

                    //terrain_ids.insert(tid);
                    let t = (collision.t - EPSILON).min(1.0).max(0.0);
                    prev_pos_value += delta * t;

                    vel_value = collision.velocity;
                    delta = (vel_value + collision.offset) * (1.0 - t);
                    pos_value = prev_pos_value + delta;
                    if let Some(state) = &mut state {
                        if collision.typ == CollisionType::Floor {
                            state.on_ground = true;
                        }
                    }
                }
                None => {
                    pos.0 = pos_value;
                    vel.0 = vel_value;
                    return;
                }
            }
        }
        pos.0 = prev_pos_value;
        vel.0 = initial_vel_value;
        log::info!("Collision retries exhausted");
    }
}