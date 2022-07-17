use crate::game::{GameState, SystemLabels, run_if_tick_elapsed};

mod components;
mod terrain;
mod collision;

pub use bevy::prelude::*;

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
            .with_system(collide_cylinders_with_terrain
                .label(SystemLabels::PhysicsCollide)
                .after(SystemLabels::PhysicsMove)
            )
            .with_system(cast_cylinders_on_terrain
                .label(SystemLabels::PhysicsCast)
                .after(SystemLabels::PhysicsCollide)
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


fn collide_cylinders_with_terrain(
    terrain_entity: Query<&Terrain>,
    mut collidable_entities: Query<(
        &mut Position,
        &PreviousPosition,
        &CylinderShape,
        &mut Velocity,
        &mut WallState
    )>
) {
    log::debug!("(SYSTEM) collide_cylinders_with_terrain");

    const COLLISION_RETRIES: usize = 8;

    // Gets terrain to collide with
    let terrain: &Terrain = match terrain_entity.iter().next() {
        Some(entity) => entity,
        None => return
    };

    // For all collidable entities
    for (mut pos, prev_pos, shape, mut vel, mut state) in collidable_entities.iter_mut() {

        // Performs pushing logic
        let mut cylinder = CylinderCollider {
            center: prev_pos.0,
            radius: shape.radius,
            half_height: shape.half_height
        };
        let coll_info = coll_cyl_with_retries(
            &terrain,
            &mut cylinder,
            vel.0,
            COLLISION_RETRIES
        );
        if let Some(coll_info) = coll_info {
            pos.0 = coll_info.position;
            vel.0 = coll_info.velocity;
            state.on_ground = coll_info.on_ground
        }
    }
}

/// Performs "downward-casting" logic to keep physics entities stuck to the ground when going down slopes, stairs, etc.
fn cast_cylinders_on_terrain(
    terrain_entity: Query<&Terrain>,
    mut collidable_entities: Query<(
        &mut Position,
        &CylinderShape,
        &mut WallState
    )>
) {
    log::debug!("(SYSTEM) cast_cylinders_on_terrain");

    const COLLISION_RETRIES: usize = 8;

    // Gets terrain to collide with
    let terrain: &Terrain = match terrain_entity.iter().next() {
        Some(entity) => entity,
        None => return
    };

    // For all collidable entities
    for (mut pos, shape, mut state) in collidable_entities.iter_mut() {
        if !state.on_ground && !state.prev_on_ground {
            continue;
        }

        // Performs pushing logic
        let mut cylinder = CylinderCollider {
            center: pos.0,
            radius: shape.radius,
            half_height: shape.half_height
        };
        let coll_info = coll_cyl_with_retries(
            &terrain,
            &mut cylinder,
            Vec3::new(0.0, -4.0, 0.0),
            COLLISION_RETRIES
        );
        if let Some(coll_info) = coll_info {
            pos.0 = coll_info.position;
            state.on_ground = coll_info.on_ground;
        }
    }
}


fn coll_cyl_with_retries(
    terrain: &Terrain,
    cyl: &mut CylinderCollider,
    mut delta: Vec3,
    retries: usize
) -> Option<CollisionInfo> {
    
    const EPSILON: f32 = 0.01;
    if retries == 0 {
        panic!("Invalid number of retries {}", retries);
    }
    let mut result = None;
    let mut on_ground = false;

    for i in 0..retries {
        
        // Finds collision with terrain and cylinder
        let coll = terrain.collide_with_cylinder(cyl, delta);

        // If there was a collision, gather information about it and prepare for the next retry
        match coll {
            Some((collision, _)) => {

                // If on the last retry, we'll want to ignore this collision
                if i == retries - 1 {
                    println!("Retries exhausted on push");
                    break;
                }
                
                // Massages t value
                let t = (collision.t - EPSILON).min(1.0).max(0.0);

                // Applies collision to previous position
                cyl.center += delta * t;
                delta = (collision.velocity + collision.offset) * (1.0 - t);
                if collision.typ == CollisionType::Floor {
                    on_ground = true;
                }

                // Writes to result
                result = Some(CollisionInfo {
                    position: cyl.center + delta,
                    prev_position: cyl.center,
                    velocity: collision.velocity,
                    on_ground,
                });
            }
            None => {}
        };
    }

    // Returns the result, if any
    result
}