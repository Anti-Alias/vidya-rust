use std::time::{Duration, Instant};

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
        app
            .add_plugins(DefaultPlugins)
            .add_state(AppState::AppStarting)
            .insert_resource(AppConfig {
                side: Side::Client,
                ticks_per_second: 60
            })
            .add_startup_system_set(SystemSet::new()
                .with_system(start_app)
                .with_system(configure_resources)
            )
            .add_system(update_tick_timer.label(AppLabel::Tick));
    }
    fn name(&self) -> &str { "vidya_plugin" }
}

/// Responsible for keeping track of in-game ticks at 60tps
pub struct TickTimer(Timer);
impl TickTimer {
    pub fn times_finished(&self) -> u32 { self.0.times_finished() }
    pub fn finished(&self) -> bool { self.0.finished() }
    pub fn t(&self) -> f32 {
        let duration = self.0.duration();
        let elapsed = self.0.elapsed();
        let t = elapsed.as_secs_f32() / duration.as_secs_f32();
        t
    }
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// Configuration of the application as a whole
pub struct AppConfig {
    pub side: Side,
    pub ticks_per_second: u64
}

fn start_app(mut app_state: ResMut<State<AppState>>) {
    log::debug!("Entered system 'start_app'");
    app_state.set(AppState::AppRunning).unwrap();
}

/// Configures resources based on the configurations in the [`AppConfig`] resource
fn configure_resources(
    config: Res<AppConfig>,
    mut commands: Commands
) {
    let tick_time_ms = Duration::from_millis(1000 / config.ticks_per_second);
    commands.insert_resource(TickTimer(Timer::new(tick_time_ms, true)));
}

/// Updates main tick timer
fn update_tick_timer(time: Res<Time>, mut tick_timer: ResMut<TickTimer>) {
    tick_timer.0.tick(time.delta());
    //log::info!("Updated tick timer");
}