use bevy::prelude::*;
use bevy::render::camera::Camera3d;

use crate::app::{AppState, SystemLabels, tick_elapsed};
use crate::physics::{Velocity, Friction, PreviousPosition, Position};

use std::f32::consts::SQRT_2;


pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(AppState::AppRunning)
            .with_run_criteria(tick_elapsed)
            .label(SystemLabels::CameraUpdate)
            .after(SystemLabels::TickStart)
            .after(SystemLabels::PhysicsMove)
            .after(SystemLabels::PhysicsCollide)
            .with_system(camera_target)
        );
    }
}

/// Bundle of camera components
#[derive(Bundle)]
pub struct CameraBundle {
    #[bundle]
    ortho_bundle: OrthographicCameraBundle<Camera3d>,
    position: Position,
    prev_position: PreviousPosition,
    velocity: Velocity,
    friction: Friction,
    settings: CameraTargetSettings
}
impl CameraBundle {
    pub fn new(
        ortho_bundle: OrthographicCameraBundle<Camera3d>,
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
/// There should only be 1 entity with this marker at a time.
#[derive(Component)]
pub struct Targetable;

#[derive(Component)]
pub struct CameraTargetSettings { pub distance: f32 }

pub fn camera_target(
    targetable: Query<&Position, (With<Targetable>, Without<Camera>)>,
    mut camera: Query<(&mut Position, &CameraTargetSettings), With<Camera>>
) {
    log::debug!("(SYSTEM) camera_target");
    // Gets target and camera
    let target_pos = match targetable.get_single() {
        Ok(pos) => pos,
        Err(_) => return
    };
    let (mut camera_pos, camera_settings) = match camera.get_single_mut() {
        Ok(result) => result,
        Err(_) => return
    };
    
    // Sets camera's position as the target's position
    let dist = camera_settings.distance;
    let yz =dist*SQRT_2;
    camera_pos.0 = target_pos.0 + Vec3::new(0.0, yz, yz);
}