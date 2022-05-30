use crate::app::{AppState, SystemLabels, tick_elapsed};

mod movement;
mod terrain;
mod collision;

pub use bevy::prelude::*;

pub use terrain::*;
pub use collision::*;
pub use movement::*;
pub use movement::SizeCylinder;

/// Plugin that adds physics components and terrain collision
pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Gravity::default());
        app.add_system_set(SystemSet::on_update(AppState::AppRunning)
            .with_run_criteria(tick_elapsed)
            .after(SystemLabels::TickStart)
            .with_system(apply_gravity
                .label(SystemLabels::PhysicsGravity)
                .after(SystemLabels::Logic)
            )
            .with_system(apply_friction
                .label(SystemLabels::PhysicsFriction)
                .after(SystemLabels::Logic)
                .after(SystemLabels::PhysicsGravity)
            )
            .with_system(sync_previous_state
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


const COLLISION_RETRIES: u32 = 10;
fn collide_with_terrain(
    terrain_entity: Query<&Terrain>,
    mut collidable_entities: Query<(&mut Position, &PreviousPosition, &SizeCylinder, &mut Velocity)>
) {
    log::debug!("(SYSTEM) collide_with_terrain");

    // Gets terrain to collide with
    let terrain = match terrain_entity.iter().next() {
        Some(entity) => entity,
        None => return
    };

    // For all collidable entities
    for (mut pos, prev_pos, size, mut vel) in collidable_entities.iter_mut() {
        let mut pos_value = pos.0;
        let mut prev_pos_value = prev_pos.0;
        let mut vel_value = vel.0;

        // For N retries...
        for _ in 0..COLLISION_RETRIES {
            let cylinder = CylinderCollider {
                center: prev_pos_value,
                radius: size.radius,
                half_height: size.half_height
            };
            let coll = terrain.collide_with_cylinder(&cylinder, vel_value);
            match coll {
                Some(coll) => {
                    prev_pos_value = prev_pos_value + vel_value * coll.t;
                    vel_value = coll.velocity;
                    pos_value = prev_pos_value + vel_value;
                }
                None => {
                    pos.0 = pos_value;
                    vel.0 = vel_value;
                    return;
                }
            }
        }
        println!("Retries exhausted!");
    }
}