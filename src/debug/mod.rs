use bevy::prelude::*;

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, _: &mut App) {

    }
}

/// Component that allows for 3D movement using WASD for X/Z movement, and CTRL and Shift for Y movement
pub struct Floater;