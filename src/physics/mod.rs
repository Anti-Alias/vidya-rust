use crate::app::{AppState, AppLabel, tick_elapsed};

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
            .after(AppLabel::TickStart)
            .with_system(apply_gravity
                .label(AppLabel::PhysicsGravity)
                .after(AppLabel::Logic)
            )
            .with_system(apply_friction
                .label(AppLabel::PhysicsFriction)
                .after(AppLabel::Logic)
                .after(AppLabel::PhysicsGravity)
            )
            .with_system(sync_previous_state
                .label(AppLabel::PhysicsSync)
                .before(AppLabel::PhysicsMove)
                .after(AppLabel::Logic)
            )
            .with_system(apply_velocity
                .label(AppLabel::PhysicsMove)
                .after(AppLabel::Logic)
                .after(AppLabel::PhysicsFriction)
            )
            .with_system(collide_with_terrain
                .label(AppLabel::PhysicsCollide)
                .after(AppLabel::PhysicsMove)
            )
        );
    }
}

fn collide_with_terrain(
    terrain_entity: Query<&Terrain>,
    mut collidable_entities: Query<(&mut Position, &PreviousPosition, &SizeCylinder)>
) {
    log::debug!("(SYSTEM) collide_with_terrain");
    let terrain = match terrain_entity.iter().next() {
        Some(entity) => entity,
        None => return
    };
    for (mut pos, prev_pos, size) in collidable_entities.iter_mut() {
        let cylinder = CylinderCollider {
            center: pos.0,
            radius: size.radius,
            half_height: size.half_height
        };
        let delta = pos.0 - prev_pos.0;
        let coll = terrain.collide_with_cylinder(&cylinder, delta);
        match coll {
            Some(_) => {
                println!("Collision!!!");
            }
            None => {
                //println!("No collision...");
            }
        }
    }
}