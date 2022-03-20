use std::time::Duration;

use crate::animation::AnimationPlugin;
use crate::graphics::GraphicsPlugin;
use crate::platformer::PlatformerPlugin;
use crate::player::PlayerPlugin;
use crate::sprite::SpritePlugin;
use crate:: {
    camera::CameraPlugin,
    map::MapPlugin,
    //debug::DebugPlugin,
    physics::PhysicsPlugin,
};

use bevy::app::PluginGroupBuilder;
use bevy::ecs::schedule::ShouldRun;
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
        //builder.add(DebugPlugin);
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
            .add_system_set(SystemSet::on_update(AppState::AppRunning)
                .with_system(update_partial_ticks.label(AppLabel::TickStart))
            )
            .add_startup_system_set(SystemSet::new()
                .with_system(configure_app)
                .with_system(start_app)
            );
    }
    fn name(&self) -> &str { "vidya_plugin" }
}

/// Labels used for scheduling the timing of systems in a single tick
#[derive(SystemLabel, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum AppLabel {
    /// Start of a tick.
    TickStart,

    /// Processes input and converts to signals
    Input,

    /// Performs logic, oftend dependent on signals generated in [`AppLabel::Input`] phase
    Logic,

    /// Applies gravity
    PhysicsGravity,

    /// Applies friction to velocity
    /// After Logic
    PhysicsFriction,

    /// Sets previous physics states to current physics states, IE PrevPosition, PrevSize, etc.
    /// After Logic
    PhysicsSync,

    /// Applies velocity to position
    PhysicsMove,

    /// Updates camera
    CameraUpdate,

    /// Controls animations
    ControlAnimations,

    /// Updates animations
    UpdateAnimations,

    /// Interpolates an Entity's graphics using its previous and current state
    InterpolateGraphics,

    /// Draws to sprite batches
    DrawBatches,

    /// End of tick. Prepare for next tick.
    TickEnd
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

pub struct TimestepTimer(Timer);


/// Used in graphics to interpolate between previous and current state.
/// Allows for variable refresh rates
#[derive(Debug, Default, Clone)]
pub struct PartialTicks { timer: Timer }
impl PartialTicks {

    /// Creates new PartialTicks struct
    fn new(timestep_secs: f64) -> Self {
        Self { timer: Timer::new(Duration::from_secs_f64(timestep_secs), true) }
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

fn configure_app(config: Res<AppConfig>,mut commands: Commands) {
    commands.insert_resource(PartialTicks::new(config.timestep_secs));
}

fn start_app(mut app_state: ResMut<State<AppState>>) {
    log::debug!("(SYSTEM) 'start_app'");
    app_state.set(AppState::AppRunning).unwrap();
}

/// Updates the partial ticks value
fn update_partial_ticks(
    mut partial_ticks: ResMut<PartialTicks>
) {
    log::debug!("(SYSTEM) ----- update_partial_ticks ----- ");
    //let delta = time.delta();
    let delta = Duration::from_secs_f64(1.0/64.0);
    partial_ticks.timer.tick(delta);
}


/// Run criteria for when a tick has elapsed
pub fn tick_elapsed(time: Res<Time>, partial_ticks: Res<PartialTicks>) -> ShouldRun {
    let next_time = partial_ticks.timer.elapsed() + time.delta();
    if next_time >= partial_ticks.timer.duration() {
        ShouldRun::Yes
    }
    else {
        ShouldRun::No
    }
}