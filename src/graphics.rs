use bevy::prelude::*;

use crate::physics::{Position, PreviousPosition};
use crate::app::{AppState, AppLabel, PartialTicks};

pub struct GraphicsPlugin;
impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::AppRunning)
                .label(AppLabel::InterpolateGraphics)
                .after(AppLabel::TickStart)
                .after(AppLabel::PhysicsMove)
                .after(AppLabel::CameraUpdate)
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
    log::debug!("T: {}", partial_ticks.t());
    for (position, prev_position, mut transform) in query.iter_mut() {
        let src = prev_position.0;
        let dest = position.0;
        let lerped = src.lerp(dest, t).round();
        //let lerped = src.lerp(dest, 1.0).round();
        *transform = transform.with_translation(lerped);
    }
}