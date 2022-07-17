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

#[derive(Copy, Clone, Debug, PartialEq, )]
struct CollisionInfo {
    position: Vec3,
    prev_position: Vec3,
    velocity: Vec3,
    on_ground: bool
}

fn terrain_push_cylinder(
    terrain: &Terrain,
    mut cyl: CylinderCollider,
    mut delta: Vec3,
    retries: usize
) -> Option<CollisionInfo> {

    if retries == 0 {
        panic!("Invalid number of retries {}", retries);
    }

    const EPSILON: f32 = 0.01;
    let mut result = None;
    let mut on_ground = false;

    for i in 0..retries {
        
        // Collides cylinder with terrain, and gathers information
        let coll = terrain.collide_with_cylinder(&cyl, delta);
        match coll {
            Some((collision, _)) => {

                // If on the last retry, we'll want to ignore this collision
                if i == retries - 1 {
                    println!("Retries exhausted on push");
                    return None;
                }
                
                // Massages t value
                let t = (collision.t - EPSILON).min(1.0).max(0.0);

                // Applies collision to previous position
                cyl.center += delta * t;
                delta = (collision.velocity + collision.offset) * (1.0 - t);
                

                // Writes to result
                result = Some(CollisionInfo {
                    position: cyl.center + delta,
                    prev_position: cyl.center,
                    velocity: collision.velocity,
                    on_ground,
                });
            }
            None => {
                return result
            }
        };
    }

    // Required to satisfy the compiler
    None
}


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
    const COLLISION_RETRIES: usize = 8;
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
        let mut vel_value = vel.0;              // Velocity at start point
        let mut delta = vel_value;              // Change in motion
        let mut had_interactions = false;       // If false, loop will quit early
        let mut on_ground = false;

        // Pushes cylinder collider out of the terrain
        let cylinder = CylinderCollider {
            center: prev_pos.0,
            radius: size.radius,
            half_height: size.half_height
        };
        let coll_info = terrain_push_cylinder(
            &terrain,
            cylinder,
            delta,
            COLLISION_RETRIES
        );
        if let Some(coll_info) = coll_info {
            pos_value = coll_info.position;
            prev_pos_value = coll_info.prev_position;
            vel_value = coll_info.velocity;
            if coll_info.collision_type == CollisionType::Floor {
                on_ground = true;
            }
            had_interactions = true;
        }

        pos.0 = pos_value;
        vel.0 = vel_value;
        if let Some(mut state) = state {
            state.on_ground = on_ground;
        }
    }
}