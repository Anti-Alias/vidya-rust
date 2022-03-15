use crate::animation::AnimationPlugin;
use crate::platformer::PlatformerPlugin;
use crate::player::PlayerPlugin;
use crate::sprite::SpritePlugin;
use crate:: {
    camera::CameraPlugin,
    map::MapPlugin,
    debug::DebugPlugin,
    physics::PhysicsPlugin,
};

use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;

// Default plugins
pub struct VidyaPlugins;
impl PluginGroup for VidyaPlugins {
    fn build(&mut self, builder: &mut PluginGroupBuilder) {
        builder.add(VidyaCorePlugin);
        builder.add(AnimationPlugin);
        builder.add(SpritePlugin);
        builder.add(MapPlugin);
        builder.add(CameraPlugin);
        builder.add(PhysicsPlugin);
        builder.add(PlatformerPlugin);
        builder.add(DebugPlugin);
        builder.add(PlatformerPlugin);
        builder.add(PlayerPlugin);
    }
}


// Core plugin
#[derive(Default)]
pub struct VidyaCorePlugin;
impl Plugin for VidyaCorePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(DefaultPlugins)
            .add_state(AppState::AppStarting)
            .insert_resource(AppConfig { side: Side::Client })
            .add_startup_system(start_app)
        ;
    }
    fn name(&self) -> &str { "vidya_plugin" }
}

/// Labels used for scheduling the timing of systems in a single tick
#[derive(SystemLabel, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum AppLabel {
    Input,
    Logic,
    PostLogic,
    PhysicsGravity,
    PhysicsFriction,
    PhysicsVelocity,
    PhysicsSync
}

/// State of the application as a whole.
/// Dictates what systems run and when.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {

    /// No systems should run, as the application is starting
    AppStarting,

    /// App is in a free state
    AppRunning,

    /// Application stopped. No systems should run
    AppStopped,

    /// TMX file is being loaded
    MapLoadingFile,
    MapFiringEvents,
    MapHandlingEvents,
    MapSpawningEntities
}

/// Side the application is on
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Side {
    Server,
    Client
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// Configuration of the application as a whole
pub struct AppConfig {
    pub side: Side
}

fn start_app(mut app_state: ResMut<State<AppState>>) {
    log::debug!("Entered system 'start_app'");
    app_state.set(AppState::AppRunning).unwrap();
}