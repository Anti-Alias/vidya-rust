use bevy::prelude::*;

use crate::physics::{Position, PreviousPosition};
use crate::app::{AppState, AppLabel, PartialTicks};

pub struct GraphicsPlugin;
impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::AppRunning)
                .label(AppLabel::Graphics)
                .after(AppLabel::TickStart)
                .after(AppLabel::PhysicsMove)
                .with_system(sync_transform)
        );
    }
}

// Synchronizes a [`Transform`] with a [`Position`].
pub fn sync_transform(
    partial_ticks: Res<PartialTicks>,
    mut query: Query<(&Position, &PreviousPosition, &mut Transform)>
) {
    let t = partial_ticks.t();
    let mut last: Option<Vec3> = None;
    for (position, prev_position, mut transform) in query.iter_mut() {
        let a = prev_position.0;
        let b = position.0;
        let lerped = a.lerp(b, t).round();
        *transform = transform.with_translation(lerped);
        if let Some(last) = last {
            if transform.translation != last {
                log::info!("Diff: {:?}", last - transform.translation);
            }
        }
        last = Some(lerped);
    }
}