use bevy::prelude::*;
use bevy::render::texture::ImageSettings;

use crate::physics::{Position, PreviousPosition};
use crate::game::{GameState, SystemLabels};

/// Interpolates entity graphics for high refresh-rate monitors.
/// Also, defines default image settings.
pub struct GraphicsPlugin;
impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ImageSettings::default_nearest());
        app.add_system_set(
            SystemSet::on_update(GameState::GameRunning)
                .label(SystemLabels::InterpolateGraphics)
                .after(SystemLabels::PhysicsMove)
                .after(SystemLabels::PhysicsCollide)
                .after(SystemLabels::PhysicsCast)
                .after(SystemLabels::CameraUpdate)
                .with_system(interpolate_graphics)
        );
    }
}
// Synchronizes a [`Transform`] with a [`Position`].

pub fn interpolate_graphics(
    #[cfg(release)]
    partial_ticks: Res<PartialTicks>,
    mut query: Query<(&Position, &PreviousPosition, &mut Transform)>
) {
    log::debug!("(SYSTEM) interpolate_graphics");
    #[cfg(release)]
    let t = partial_ticks.t();
    #[cfg(not(release))]
    let t = 1.0;
    for (position, prev_position, mut transform) in query.iter_mut() {
        let src = prev_position.0.round();
        let dest = position.0.round();
        let lerped = src.lerp(dest, t).round();
        *transform = transform.with_translation(lerped);
    }
}