use bevy::math::Vec3Swizzles;
use bevy::prelude::*;

use crate::animation::{AnimationGroupHandle, AnimationSet};
use crate::app::{AppState, SystemLabels, tick_elapsed};
use crate::physics::{Velocity, Friction};
use crate::being::{Being, DirectionType};
use crate::state::{StateHolder, State};
use crate::util::SignalQueue;

/// Plugin for "Being" behavior
pub struct PlatformerPlugin;
impl Plugin for PlatformerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(AppState::AppRunning)
            .with_run_criteria(tick_elapsed)
            .after(SystemLabels::TickStart)
            .with_system(process_signals
                .label(SystemLabels::Logic)
                .after(SystemLabels::Input)
            )
            .with_system(control_animations
                .label(SystemLabels::ControlAnimations)
                .after(SystemLabels::Logic)
            )
        );
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
impl From<PlatformerSignal> for u32 {
    fn from(signal: PlatformerSignal) -> Self {
        match signal {
            PlatformerSignal::Move {..} => 0,
            PlatformerSignal::Jump => 1
        }
    }
}


/// Controls on-ground movement, in-air movement, and jumping
#[derive(Component, Debug)]
pub struct Platformer {
    pub top_speed: f32,
    pub signals: SignalQueue<PlatformerSignal>
}

impl Platformer {
    pub fn new(top_speed: f32) -> Self {
        Self {
            top_speed,
             signals: SignalQueue::new()
        }
    }
}

/// Holds animation groups that play/loop when the platformer performs certain actions
#[derive(Component, Debug)]
pub struct PlatformerAnimator {
    pub direction_type: DirectionType,
    pub idle_handle: AnimationGroupHandle,
    pub run_handle: AnimationGroupHandle,
    pub jump_handle: AnimationGroupHandle
}


fn process_signals(mut platformer_entities: Query<(
    &mut Platformer,
    &Friction,
    &mut Velocity,
    &mut Being,
    &mut StateHolder
)>) {
    log::debug!("(SYSTEM) process_signals");
    for (
        mut platformer,
        friction,
        mut velocity,
        mut being,
        mut state_holder
    )
    in platformer_entities.iter_mut() {

        // Process all queued signals
        let mut next_signal = platformer.signals.pop();
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
            next_signal = platformer.signals.pop();
        }

        // Updates state based on velocity
        const EPSILON: f32 = 0.1;
        if velocity.0.xz().length_squared() > EPSILON*EPSILON {
            state_holder.0 = State::Running;
        }
        else {
            state_holder.0 = State::Idle;
        }
    }
}

fn control_animations(mut platformer_entities: Query<
    (&mut AnimationSet, &PlatformerAnimator, &Being, &StateHolder),
    Changed<StateHolder>
>) {
    log::debug!("(SYSTEM) control_animations");
    for (mut animation_set, animator, being, state_holder) in platformer_entities.iter_mut() {
        match state_holder.0 {
            State::Idle => {
                animation_set.set_grouped_animation(
                    animator.idle_handle,
                    being.get_direction_index(animator.direction_type),
                    false
                ).unwrap();
            },
            crate::state::State::Running => {
                animation_set.set_grouped_animation(
                    animator.run_handle,
                    being.get_direction_index(animator.direction_type),
                    false
                ).unwrap();
            },
            crate::state::State::Jumping => {
                animation_set.set_grouped_animation(
                    animator.jump_handle,
                    being.get_direction_index(animator.direction_type),
                    false
                ).unwrap();
            },
            _ => {}
        }
    }
}