use bevy::prelude::*;

use crate::physics::{Position, PreviousPosition};
use crate::game::{GameState, SystemLabels, PartialTicks};

pub struct GraphicsPlugin;
impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::GameRunning)
                .label(SystemLabels::InterpolateGraphics)
                .after(SystemLabels::TickStart)
                .after(SystemLabels::PhysicsMove)
                .after(SystemLabels::PhysicsCollide)
                .after(SystemLabels::CameraUpdate)
                .with_system(interpolate_graphics)
        );
    }
}
// Synchronizes a [`Transform`] with a [`Position`].

pub fn interpolate_graphics(
    partial_ticks: Res<PartialTicks>,
    mut query: Query<(&Position, &PreviousPosition, &mut Transform)>
) {
    log::debug!("(SYSTEM) interpolate_graphics");
    let t = partial_ticks.t();
    for (position, prev_position, mut transform) in query.iter_mut() {
        let src = prev_position.0;
        let dest = position.0;
        let lerped = src.lerp(dest, t).round();
        *transform = transform.with_translation(lerped);
    }
}