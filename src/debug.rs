use bevy::prelude::*;

use crate::app::{AppState, TickTimer};
use crate::physics::{ Velocity };

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(SystemSet::on_update(AppState::AppRunning)
                .with_system(move_floater)
            )
        ;
    }
}

/// Component that allows for 3D movement using WASD for X/Z movement, and CTRL and Shift for Y movement
#[derive(Component, Debug, Copy, Clone, PartialEq)]
pub struct Floater { pub speed: f32 }

fn move_floater(
    tick_timer: Res<TickTimer>,
    mut query: Query<(&mut Velocity, &Floater)>,
    keys: Res<Input<KeyCode>>
) {
    for _ in 0..tick_timer.times_finished() {
        for (mut velocity, floater) in query.iter_mut() {
            if keys.pressed(KeyCode::W) { velocity.0.z -= floater.speed; }
            if keys.pressed(KeyCode::A) { velocity.0.x -= floater.speed; }
            if keys.pressed(KeyCode::S) { velocity.0.z += floater.speed; }
            if keys.pressed(KeyCode::D) { velocity.0.x += floater.speed; }
        }
    }
}