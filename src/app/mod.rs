pub use crate:: {
    camera::CameraPlugin,
    map::MapPlugin,
    debug::DebugPlugin,
    physics::PhysicsPlugin,
};

use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;

pub struct VidyaPlugins;
impl PluginGroup for VidyaPlugins {
    fn build(&mut self, builder: &mut PluginGroupBuilder) {
        builder.add(VidyaPlugin);
        builder.add(MapPlugin);
        builder.add(CameraPlugin);
        builder.add(PhysicsPlugin);
        builder.add(DebugPlugin);
    }
}


#[derive(Default)]
pub struct VidyaPlugin;
impl Plugin for VidyaPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(DefaultPlugins)
            .add_state(AppState::AppStarting)
            .add_startup_system(start_app)
        ;
    }
    fn name(&self) -> &str { "vidya_plugin" }
}

#[derive(SystemLabel, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum AppLabel {
    Input,
    Logic,
    PostLogic,
    PhysicsFriction,
    PhysicsVelocity,
    PhysicsSync
}

fn start_app(mut app_state: ResMut<State<AppState>>) {
    log::debug!("Entered system 'start_app'");
    app_state.set(AppState::AppRunning).unwrap();
}

/// High-level state of the application as a whole
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {

    // App events
    AppStarting,
    AppRunning,
    AppStopped,

    // Map events
    MapLoadingFile,
    MapFiringEvents,
    MapHandlingEvents,
    MapSpawningEntities,
    MapFinishing
}