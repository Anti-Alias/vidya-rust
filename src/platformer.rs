use bevy::math::Vec3Swizzles;
use bevy::prelude::*;

use crate::animation::{AnimationGroupHandle, AnimationSet};
use crate::game::{GameState, SystemLabels, run_if_tick_elapsed};
use crate::physics::{Velocity, Friction, Gravity, PhysicsState};
use crate::direction::{DirectionState, DirectionType};
use crate::state::{ActionState, State};
use crate::util::SignalQueue;

/// Plugin for "Platformer" behavior
pub struct PlatformerPlugin;
impl Plugin for PlatformerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(GameState::GameRunning)
            .with_run_criteria(run_if_tick_elapsed)
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
    Look { direction: f32},
    Jump
}


/// Controls on-ground movement, in-air movement, and jumping
#[derive(Component, Debug)]
pub struct Platformer {
    pub top_speed: f32,
    pub jump_height: f32,
    pub signals: SignalQueue<PlatformerSignal>
}

impl Platformer {
    pub fn new(top_speed: f32, jump_height: f32) -> Self {
        Self {
            top_speed,
            jump_height,
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


fn process_signals(
    gravity: Res<Gravity>,
    mut platformer_entities: Query<(
        &mut Platformer,
        &Friction,
        &mut Velocity,
        &mut DirectionState,
        &PhysicsState,
        &mut ActionState,
    )>)
{
    log::debug!("(SYSTEM) process_signals");
    for (
        mut platformer,
        friction,
        mut velocity,
        mut dir_state,
        physics_state,
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
                }
                PlatformerSignal::Look { direction } => {
                    dir_state.direction = direction;
                }
                PlatformerSignal::Jump => {
                    if physics_state.on_ground {
                        let g = gravity.gravity;
                        let jh = platformer.jump_height;
                        let det = g*g - 4.0 * (-jh * 2.0);
                        if det > 0.0 {
                            velocity.0.y = (-g + det.sqrt()) / 2.0;
                        }
                    }
                }
            }
            next_signal = platformer.signals.pop();
        }

        // Updates state based on velocity / groundedness
        const EPSILON: f32 = 0.1;
        if !physics_state.on_ground {
            state_holder.0 = State::Jumping;
        }
        else if velocity.0.xz().length_squared() > EPSILON*EPSILON {
            state_holder.0 = State::Running;
        }
        else {
            state_holder.0 = State::Idle;
        }
    }
}

// Controls the platformer's animation based on their current state
fn control_animations(mut platformer_entities: Query<
    (&mut AnimationSet,
    &PlatformerAnimator,
    &DirectionState,
    &ActionState),
    Changed<ActionState>
>) {
    log::debug!("(SYSTEM) control_animations");
    for (mut animation_set, animator, dir_holder, action_state) in platformer_entities.iter_mut() {
        match action_state.0 {
            State::Idle => {
                animation_set.set_grouped_animation(
                    animator.idle_handle,
                    dir_holder.get_direction_index(animator.direction_type),
                    false
                ).unwrap();
            },
            crate::state::State::Running => {
                animation_set.set_grouped_animation(
                    animator.run_handle,
                    dir_holder.get_direction_index(animator.direction_type),
                    false
                ).unwrap();
            },
            crate::state::State::Jumping => {
                animation_set.set_grouped_animation(
                    animator.jump_handle,
                    dir_holder.get_direction_index(animator.direction_type),
                    false
                ).unwrap();
            },
            _ => {}
        }
    }
}