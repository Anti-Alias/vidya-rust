use bevy::prelude::*;
use bevy::render::camera::{Projection, ScalingMode};

use crate::extensions::TransformExt;
use crate::game::{GameState, SystemLabels, run_if_tick_elapsed};
use crate::physics::{Velocity, Friction, PreviousPosition, Position};

use std::f32::consts::SQRT_2;


pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(GameState::GameRunning)
            .with_run_criteria(run_if_tick_elapsed)
            .label(SystemLabels::CameraUpdate)
            .after(SystemLabels::PhysicsMove)
            .after(SystemLabels::PhysicsCollide)
            .with_system(camera_target)
        );
    }
}

/// Bundle of camera components
#[derive(Bundle)]
pub struct GameCameraBundle {
    #[bundle]
    cam_3d_bundle: Camera3dBundle,
    position: Position,
    prev_position: PreviousPosition,
    velocity: Velocity,
    friction: Friction,
    settings: CameraTargetSettings
}
impl GameCameraBundle {
    pub fn new(
        position: Position,
        velocity: Velocity,
        friction: Friction,
        settings: CameraTargetSettings
    ) -> Self {

        // Creates camera bundle
        let width = 800.0;
        let height = 450.0;
        let cam_3d_bundle = Camera3dBundle {
            projection: Projection::Orthographic(OrthographicProjection {
                left: -width / 2.0,
                right: width / 2.0,
                bottom: -height / 2.0,
                top: height / 2.0,
                near: 1.0,
                far: 10000.0,
                scale: 0.5,
                scaling_mode: ScalingMode::WindowSize,
                ..default()
            }),
            transform: Transform::identity()
                .looking_towards(Vec3::new(0.0, -1.0, -1.0), Vec3::new(0.0, 1.0, 0.0))
                .with_scale(Vec3::new(1.0, 1.0/SQRT_2, 1.0)),
            ..default()
        };

        // Finishes GameCameraBundle
        Self {
            cam_3d_bundle,
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

// Has the camera follow an entity with a "Targettable" component
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