pub use bevy::prelude::*;

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

pub fn camera_rotate(
    mut camera: Query<(&mut Transform, &mut CameraTarget, &mut CameraTimer)>
) {
    for (mut transform, mut target, mut timer) in camera.iter_mut() {
        let pos = target.target;
        let dist = target.distance;
        let rad = timer.advance() * std::f32::consts::PI;
        *transform = transform
            .with_translation(pos + Vec3::new(f32::cos(rad)*dist, dist, f32::sin(rad)*dist))
            .looking_at(pos, Vec3::new(0.0, 1.0, 0.0));
    }
}