use bevy::prelude::*;

use crate::app::AppState;


pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(SystemSet::on_update(AppState::AppRunning)
            .with_system(camera_rotate)
        )
        ;
    }
}

#[derive(Component)]
pub struct CameraTarget {
    pub target: Vec3,
    pub distance: f32
}

#[derive(Component)]
pub struct CameraTimer {
    pub timer: f32,
    pub speed: f32
}

impl CameraTimer {
    pub fn advance(&mut self) -> f32 {
        let result = self.timer;
        self.timer += self.speed;
        result
    }
}

type RotatingCamera<'a> = (&'a mut Transform, &'a CameraTarget, &'a mut CameraTimer);

pub fn camera_rotate(
    mut camera: Query<RotatingCamera>
) {
    for (mut transform, target, mut timer) in camera.iter_mut() {
        let pos = target.target;
        let dist = target.distance;
        let rad = timer.advance() * std::f32::consts::PI;
        *transform = transform
            .with_translation(pos + Vec3::new(f32::cos(rad)*dist, dist, f32::sin(rad)*dist))
            .looking_at(pos, Vec3::new(0.0, 1.0, 0.0));
    }
}