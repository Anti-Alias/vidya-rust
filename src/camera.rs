use bevy::prelude::*;

use crate::app::{AppState, AppLabel, tick_elapsed};
use crate::physics::{Velocity, Friction, PreviousPosition, Position};


pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(AppState::AppRunning)
            .with_run_criteria(tick_elapsed)
            .label(AppLabel::Graphics)
            .after(AppLabel::TickStart)
            .after(AppLabel::PhysicsMove)
            .with_system(camera_move)
        );
        app.add_system_set(SystemSet::on_update(AppState::AppRunning)
            .with_run_criteria(tick_elapsed)
            .label(AppLabel::PostGraphics)
            .after(AppLabel::TickStart)
            .after(AppLabel::Graphics)
            .with_system(push_camera_back)
        );
    }
}

/// Bundle of camera components
#[derive(Bundle)]
pub struct CameraBundle {
    #[bundle]
    ortho_bundle: OrthographicCameraBundle,
    position: Position,
    prev_position: PreviousPosition,
    velocity: Velocity,
    friction: Friction,
    settings: CameraTargetSettings
}
impl CameraBundle {
    pub fn new(
        ortho_bundle: OrthographicCameraBundle,
        position: Position,
        velocity: Velocity,
        friction: Friction,
        settings: CameraTargetSettings
    ) -> Self {
        Self {
            ortho_bundle,
            position,
            prev_position: PreviousPosition(position.0),
            velocity,
            friction,
            settings
        }
    }
}

/// Tag component that marks the entity as "targettable" by the camera.
/// There should only be 1 entity with this marker.
#[derive(Component)]
pub struct Targetable;

#[derive(Component)]
pub struct CameraTargetSettings {
    pub distance: f32
}

pub fn camera_move(
    targetable: Query<&Position, (With<Targetable>, Without<Camera>)>,
    mut camera: Query<&mut Position, With<Camera>>
) {

    // Gets target and camera
    let target_pos = match targetable.get_single() {
        Ok(pos) => pos,
        Err(_) => return
    };
    let mut camera_pos = match camera.get_single_mut() {
        Ok(result) => result,
        Err(_) => return
    };
    
    // Sets camera's position as the target's position
    camera_pos.0 = target_pos.0;
}

pub fn push_camera_back(mut camera: Query<(&mut Transform, &CameraTargetSettings), With<Camera>>) {
    for (mut trans, settings) in camera.iter_mut() {
        trans.translation += Vec3::new(0.0, 512.0, 512.0);
        log::info!("Cam translation: {:?}", trans.translation);
    }
}