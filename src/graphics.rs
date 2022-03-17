use bevy::prelude::*;

use crate::{physics::{Position, PreviousPosition}, app::{AppState, AppLabel, PartialTicks}};

pub struct GraphicsPlugin;
impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(AppState::AppRunning)
            .with_system(sync_transform)
            .after(AppLabel::PhysicsVelocity)
        );
    }
}

// Synchronizes a [`Transform`] with a [`Position`].
pub fn sync_transform(
    partial_ticks: Res<PartialTicks>,
    mut query: Query<(&Position, &PreviousPosition, &mut Transform)>
) {
    let t = partial_ticks.t();
    for (position, prev_position, mut transform) in query.iter_mut() {
        let a = prev_position.0;
        let b = position.0;
        let lerped = a.lerp(b, t);
        let lerped = Vec3::new(
            lerped.x.round(),
            lerped.y.round(),
            lerped.z.round()
        );
        *transform = transform.with_translation(lerped);
    }
}