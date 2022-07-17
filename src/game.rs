use std::time::Duration;

use crate::animation::AnimationPlugin;
#[cfg(feature = "debug")]
use crate::debug::DebugPlugin;
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

/// Group of plugins that vidya uses
pub struct GamePlugins;
impl PluginGroup for GamePlugins {
    fn build(&mut self, builder: &mut PluginGroupBuilder) {
        builder.add(GraphicsPlugin);    // This needs to appear before CorePlugin. Otherwise, images will come with a linear sampler by default.
        builder.add(CorePlugin);
        builder.add(SpritePlugin);
        builder.add(AnimationPlugin);
        builder.add(MapPlugin);
        builder.add(CameraPlugin);
        builder.add(PhysicsPlugin);
        builder.add(PlatformerPlugin);
        #[cfg(feature = "debug")]
        builder.add(DebugPlugin);
        builder.add(PlayerPlugin);
    }
}

// Core plugin
#[derive(Default)]
pub struct CorePlugin;
impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(DefaultPlugins)
            .add_state(GameState::GameRunning)
            .init_resource::<GameConfig>()
            .add_system_to_stage(CoreStage::PreUpdate, update_partial_ticks)
            .add_startup_system(configure_app);
    }
    fn name(&self) -> &str { "vidya_plugin" }
}

/// Labels used for scheduling the timing of systems in a single tick
#[derive(SystemLabel, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum SystemLabels {
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

    /// Collides objects with each other after movement happens
    PhysicsCollide,

    /// Casts colliders down
    PhysicsCast,

    /// Updates camera
    CameraUpdate,

    /// Controls entity state
    ControlState,

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
/// Dictates what systems run and when during the lifecycle of the game.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {

    /// Game started and is in a free state
    GameRunning,

    /// Application stopped. No systems should run
    GameStopped,

    /// TMX file is being loaded
    MapLoadingFile,

    /// Map collision and/or graphics are being constructed
    MapConstructing,

    /// Map is being spawned
    MapSpawning
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
pub struct PartialTicks {
    timer: Timer
}

impl PartialTicks {

    /// Creates new PartialTicks struct
    fn new(duration: Duration) -> Self {
        Self {
            timer: Timer::new(duration, true),
        }
    }

    /// Advances timer
    fn tick(&mut self, duration: Duration) {
        self.timer.tick(duration);
    }

    /// T value between 0.0 and 1.0 used for lerping graphics
    pub fn t(&self) -> f32 {
        self.timer.elapsed().as_secs_f32() / self.timer.duration().as_secs_f32()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
/// Configuration of the application as a whole
pub struct GameConfig {
    pub side: Side,
    pub timestep_secs: f64
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            side: Side::Client,
            timestep_secs: 1.0/60.0
        }
    }
}

fn configure_app(config: Res<GameConfig>,mut commands: Commands) {
    commands.insert_resource(PartialTicks::new(Duration::from_secs_f64(config.timestep_secs)));
}

/// Updates the partial ticks value
fn update_partial_ticks(
    time: Res<Time>,
    mut partial_ticks: ResMut<PartialTicks>
) {
    log::debug!("(SYSTEM) ----- update_partial_ticks ----- ");
    partial_ticks.tick(time.delta());
}


/// Run criteria for when a tick has elapsed
pub fn run_if_tick_elapsed(
    #[cfg(release)]
    partial_ticks: ResMut<PartialTicks>
) -> ShouldRun {
    #[cfg(release)]
    if partial_ticks.times_finished() != 0 {
        ShouldRun::Yes
    }
    else {
        ShouldRun::No
    }
    #[cfg(not(release))]
    ShouldRun::Yes
}