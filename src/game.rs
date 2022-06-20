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

/// Group of plugins that vidya uses
pub struct GamePlugins;
impl PluginGroup for GamePlugins {
    fn build(&mut self, builder: &mut PluginGroupBuilder) {
        builder.add(CorePlugin);
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
pub struct CorePlugin;
impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(DefaultPlugins)
            .add_state(GameState::GameStarting)
            .init_resource::<GameConfig>()
            .add_system_set(SystemSet::on_in_stack_update(GameState::GameRunning)
                .with_system(update_partial_ticks.label(SystemLabels::TickStart))
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
pub enum SystemLabels {
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

    /// Collides objects with each other after movement happens
    PhysicsCollide,

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
/// Dictates what systems run and when during the lifecycle of the game.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {

    /// No systems should run, as the game is starting
    GameStarting,

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
pub struct PartialTicks { timer: BiasedTimer }
impl PartialTicks {

    /// Creates new PartialTicks struct
    fn new(timestep_secs: f64) -> Self {
        Self { timer: BiasedTimer::new(Duration::from_secs_f64(timestep_secs)) }
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
    commands.insert_resource(PartialTicks::new(config.timestep_secs));
}

fn start_app(mut app_state: ResMut<State<GameState>>) {
    log::debug!("(SYSTEM) 'start_app'");
    app_state.set(GameState::GameRunning).unwrap();
}

/// Updates the partial ticks value
fn update_partial_ticks(
    time: Res<Time>,
    mut partial_ticks: ResMut<PartialTicks>
) {
    log::debug!("(SYSTEM) ----- update_partial_ticks ----- ");
    let delta = time.delta();
    partial_ticks.timer.tick(delta);
}


/// Run criteria for when a tick has elapsed
pub fn tick_elapsed(time: Res<Time>, partial_ticks: Res<PartialTicks>) -> ShouldRun {
    let next_time = partial_ticks.timer.elapsed() + modified_delta(time.delta());
    if next_time >= partial_ticks.timer.duration() {
        ShouldRun::Yes
    }
    else {
        ShouldRun::No
    }
}

/// Timer that is biased towards 60 and 120 hz.
/// Better for pixel-perfect movements at those refresh rates.
#[derive(Debug, Default, Clone)]
pub struct BiasedTimer {
    timer: Timer,
    elapsed: Duration,
    modified_elapsed: Duration
}
impl BiasedTimer {

    /// Creates new biased timer
    pub fn new(duration: Duration) -> Self {
        Self {
            timer: Timer::new(duration, true),
            elapsed: Duration::new(0, 0),
            modified_elapsed: Duration::new(0, 0)
        }
    }

    pub fn tick(&mut self, delta: Duration) {
        let m_delta = modified_delta(delta);
        self.timer.tick(m_delta);
        self.elapsed += delta;
        self.modified_elapsed += m_delta;
    }

    pub fn elapsed(&self) -> Duration { self.timer.elapsed() }

    pub fn duration(&self) -> Duration { self.timer.duration() }
}

/// If delta is close to 1/60, just return 1/60.
/// If delta is close to 1/120, just return 1/120.
/// Else, return delta
fn modified_delta(delta: Duration) -> Duration {
    let threshold = Duration::from_millis(3);
    let hz60 = Duration::from_secs_f64(1.0/60.0);
    let diff60 = if delta < hz60 { hz60 - delta } else { delta - hz60 };
    if diff60 < threshold {
        hz60
    }
    else {
        let hz120 = Duration::from_secs_f64(1.0/120.0);
        let diff120 = if delta < hz120 { hz120 - delta } else { delta - hz120 };
        if diff120 < threshold {
            hz120
        }
        else {
            delta
        }
    }
}