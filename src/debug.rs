use bevy::prelude::*;
use bevy::render::camera::Projection;

use crate::game::{GameState, run_if_tick_elapsed};

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
    query: Query<&mut Projection, With<Camera>>,
    keys: Res<Input<KeyCode>>,
    mut toggle: Local<bool>
) {
    log::debug!("(SYSTEM) change_camera_perspective");
    if keys.pressed(config.debug_hotkey) {
        if keys.just_pressed(config.camera_toggle_hotkey) {
            *toggle = !*toggle;
            if *toggle {
                println!("Switched to perspective projection");
            }
            else {
                println!("Switched to orthographic projection");
            }
        }
    }
}