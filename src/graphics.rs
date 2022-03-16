use bevy::prelude::*;

use crate::{physics::{Position, PreviousPosition}, app::AppState};

pub struct GraphicsPlugin;
impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(AppState::AppRunning).with_system(sync_transform));
    }
}

// Synchronizes a [`Transform`] with a [`Position`].
pub fn sync_transform(mut query: Query<(&Position, &PreviousPosition, &mut Transform)>) {
    for (position, _prev_position, mut transform) in query.iter_mut() {
        let position = Vec3::new(
            position.0.x.round(),
            position.0.y.round(),
            position.0.z.round()
        );
        *transform = transform.with_translation(position);
    }
}