use bevy::prelude::*;

use crate::game::{GameState, tick_elapsed};
use crate::physics::{ Velocity };

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(SystemSet::on_update(GameState::GameRunning)
                .with_run_criteria(tick_elapsed)
                .with_system(move_floater)
            )
        ;
    }
}

/// Component that allows for 3D movement using WASD for X/Z movement, and CTRL and Shift for Y movement
#[derive(Component, Debug, Copy, Clone, PartialEq)]
pub struct Floater { pub speed: f32 }

fn move_floater(
    mut query: Query<(&mut Velocity, &Floater)>,
    keys: Res<Input<KeyCode>>
) {
    log::debug!("(SYSTEM) move_floater");
    for (mut velocity, floater) in query.iter_mut() {
        if keys.pressed(KeyCode::W) { velocity.0.z -= floater.speed; }
        if keys.pressed(KeyCode::A) { velocity.0.x -= floater.speed; }
        if keys.pressed(KeyCode::S) { velocity.0.z += floater.speed; }
        if keys.pressed(KeyCode::D) { velocity.0.x += floater.speed; }
    }
}