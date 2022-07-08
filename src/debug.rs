use bevy::prelude::*;
use bevy::render::camera::Projection;

use crate::{game::{GameState, run_if_tick_elapsed}, camera::{MainCamera, CameraTargetSettings}};

pub struct DebugConfig {
    debug_hotkey: KeyCode,
    camera_toggle_hotkey: KeyCode
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            debug_hotkey: KeyCode::LAlt,
            camera_toggle_hotkey: KeyCode::C
        }
    }
}

// Local value to store old camera value.
// Used when camera is being toggled between perspective and orthographic
struct OldCameraValues {
    projection: Projection,
    settings: CameraTargetSettings
}

pub struct DebugPlugin;
impl Plugin for DebugPlugin {

    fn build(&self, app: &mut App) {
        app
            .init_resource::<DebugConfig>()
            .add_system_set(SystemSet::on_update(GameState::GameRunning)
                .with_run_criteria(run_if_tick_elapsed)
                .with_system(change_camera_perspective)
            )
        ;
    }
}
fn change_camera_perspective(
    config: Res<DebugConfig>,
    mut query: Query<(&mut Projection, &mut CameraTargetSettings), With<MainCamera>>,
    keys: Res<Input<KeyCode>>,
    mut old_cam: Local<Option<OldCameraValues>>
) {
    log::debug!("(SYSTEM) change_camera_perspective");
    if keys.pressed(config.debug_hotkey) && keys.just_pressed(config.camera_toggle_hotkey) {
        for (mut cam_proj, mut cam_settings) in query.iter_mut() {
            match &*old_cam {
                Some(old_value) => {
                    let temp = OldCameraValues {
                        projection: cam_proj.clone(),
                        settings: cam_settings.clone()
                    };
                    *cam_proj = old_value.projection.clone();
                    *cam_settings = old_value.settings.clone();
                    *old_cam = Some(temp);
                },
                None => {
                    *old_cam = Some(OldCameraValues {
                        projection: cam_proj.clone(),
                        settings: cam_settings.clone()
                    });
                    *cam_proj = Projection::Perspective(PerspectiveProjection {
                        fov: 45.0,
                        aspect_ratio: 16.0/9.0,
                        near: 0.1,
                        far: 10000.0,
                    });
                    cam_settings.distance = 128.0
                }
            }
            
        }
    }
}