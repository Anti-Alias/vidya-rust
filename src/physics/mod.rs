use crate::app::{AppState, AppLabel, tick_elapsed};

mod movement;
mod terrain;
mod collision;

pub use movement::*;
pub use terrain::*;
pub use collision::*;

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
            // .with_system(collide_with_terrain
            //     .label(AppLabel::PhysicsCollide)
            //     .after(AppLabel::PhysicsMove)
            // )
        );
    }
}