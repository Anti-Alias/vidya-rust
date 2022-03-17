use std::time::Duration;

use crate::animation::AnimationPlugin;
use crate::graphics::GraphicsPlugin;
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
        builder.add(GraphicsPlugin);
        builder.add(SpritePlugin);
        builder.add(AnimationPlugin);
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
        let timestep_secs = 1.0/60.0;
        app
            .add_plugins(DefaultPlugins)
            .add_state(AppState::AppStarting)
            .insert_resource(AppConfig {
                side: Side::Client,
                timestep_secs
            })
            .insert_resource(PartialTicks::new(timestep_secs))
            .add_system(update_partial_ticks.label(AppLabel::Tick))
            .add_startup_system_set(SystemSet::new()
                .with_system(start_app)
            );
    }
    fn name(&self) -> &str { "vidya_plugin" }
}

/// Labels used for scheduling the timing of systems in a single tick
#[derive(SystemLabel, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum AppLabel {
    /// Updates tick timer
    Tick,

    /// Processes input and converts to signals
    Input,

    /// Performs logic, oftend dependent on signals generated in [`AppLabel::Input`] phase
    Logic,

    /// Applies friction to velocity
    PhysicsFriction,

    /// Applies velocity to position
    PhysicsVelocity,

    /// Syncs position with graphics transforms
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


/// Used in graphics to interpolate between previous and current state.
/// Allows for variable refresh rates
#[derive(Debug, Default, Clone)]
pub struct PartialTicks { timer: Timer }
impl PartialTicks {

    /// Creates new PartialTicks struct
    fn new(timestep_secs: f64) -> Self {
        Self {
            timer: Timer::new(Duration::from_secs_f64(timestep_secs), true)
        }
    }

    /// T value between 0.0 and 1.0 used for lerping graphics
    pub fn t(&self) -> f32 {
        self.timer.elapsed().as_secs_f32() / self.timer.duration().as_secs_f32()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
/// Configuration of the application as a whole
pub struct AppConfig {
    pub side: Side,
    pub timestep_secs: f64
}

fn start_app(mut app_state: ResMut<State<AppState>>) {
    log::debug!("Entered system 'start_app'");
    app_state.set(AppState::AppRunning).unwrap();
}

/// Updates the partial ticks value
fn update_partial_ticks(time: Res<Time>, mut partial_ticks: ResMut<PartialTicks>) {
    let delta = time.delta();
    partial_ticks.timer.tick(delta);
}
