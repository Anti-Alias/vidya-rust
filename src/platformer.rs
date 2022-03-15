use std::collections::VecDeque;

use bevy::prelude::*;

use crate::app::{AppState, AppLabel};
use crate::physics::{Velocity, Friction};
use crate::being::Being;

/// Plugin for "Being" behavior
pub struct PlatformerPlugin;
impl Plugin for PlatformerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(SystemSet::on_update(AppState::AppRunning)
                .with_system(process_signals.label(AppLabel::Logic))
            )
        ;
    }
}

/// Signal that an entity can receive.
/// Represents an instruction to carry out.
/// Either converted from user input, or emitted directly from an AI.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PlatformerSignal {
    Move { direction: f32 },
    Jump
}

#[derive(Component, Debug)]
pub struct Platformer {
    pub top_speed: f32,
    pub signals: VecDeque<PlatformerSignal>
}

impl Platformer {
    pub fn new(top_speed: f32) -> Self {
        Self {
            top_speed,
             signals: VecDeque::new()
        }
    }
}

/// State a being is in
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum State {
    Idle,
    Running,
    Jumping,
    Attacking
}

impl Default for State {
    fn default() -> Self { Self::Idle }
}

fn process_signals(mut platformer_entities: Query<(&mut Platformer, &Friction, &mut Velocity, &mut Being)>) {

    // For all platformer entities...
    for (mut platformer, friction, mut velocity, mut being) in platformer_entities.iter_mut() {

        // Process all queued signals
        let mut next_signal = platformer.signals.pop_front();
        while let Some(signal) = next_signal {
            match signal {
                PlatformerSignal::Move { direction } => {
                    let speed = platformer.top_speed / friction.xz - platformer.top_speed;
                    let vel = speed * Vec2::new(f32::cos(direction), -f32::sin(direction));
                    velocity.0.x += vel.x;
                    velocity.0.z += vel.y;
                    being.direction = direction;
                }
                PlatformerSignal::Jump => {
                    log::info!("Jumping!!!");
                }
            }
            next_signal = platformer.signals.pop_front();
        }
    }
}

// ts = 10
// f = 0.9
// (ts + s) * f = ts
// ts + s = ts / f
// s = ts/f - ts