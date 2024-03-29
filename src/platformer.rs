use bevy::math::Vec3Swizzles;
use bevy::prelude::*;

use crate::animation::{AnimationGroupHandle, AnimationSet};
use crate::game::{GameState, SystemLabels, run_if_tick_elapsed};
use crate::physics::{Velocity, Friction, Gravity, WallState};
use crate::direction::{DirectionState, DirectionType};
use crate::state::{ActionState, State};
use crate::util::SignalQueue;

/// Plugin for "Platformer" behavior
pub struct PlatformerPlugin;
impl Plugin for PlatformerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(GameState::GameRunning)
            .with_run_criteria(run_if_tick_elapsed)
            .with_system(control_animations
                .label(SystemLabels::ControlAnimations)
                .after(SystemLabels::ControlState)
            )
            .with_system(control_state
                .label(SystemLabels::ControlState)
                .after(SystemLabels::PhysicsCollide)
                .after(SystemLabels::PhysicsCast)
            )
            .with_system(process_signals
                .label(SystemLabels::Logic)
                .after(SystemLabels::Input)
            )
            // .with_system(log_position.after(SystemLabels::PhysicsCollide))
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
        &mut WallState
    )>)
{
    log::debug!("(SYSTEM) process_signals");
    for (
        mut platformer,
        friction,
        mut velocity,
        mut dir_state,
        mut wall_state
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
                    if wall_state.on_ground {
                        let g = gravity.gravity;
                        let jh = platformer.jump_height;
                        let det = g*g - 4.0 * (-jh * 2.0);
                        if det > 0.0 {
                            velocity.0.y = (-g + det.sqrt()) / 2.0;
                            wall_state.jump();
                        }
                    }
                }
            }
            next_signal = platformer.signals.pop();
        }
    }
}

/// Updates the platformer's state based on physics state and velocity.
fn control_state(mut query: Query<
    (
        &WallState,
        &Velocity,
        &mut ActionState,
    ),
    With<Platformer>>
) {
    log::debug!("(SYSTEM) control_state");
    for (physics_state, velocity, mut action_state) in query.iter_mut() {
        const RUN_SPEED: f32 = 0.05;
        if !physics_state.on_ground {
            action_state.0 = State::Jumping;
        }
        else if velocity.0.xz().length_squared() > RUN_SPEED*RUN_SPEED {
            action_state.0 = State::Running;
        }
        else {
            action_state.0 = State::Idle;
        }
    }
}

// Controls the platformer's animation based on their current state
fn control_animations(mut platformer_entities: Query<
    (
        &mut AnimationSet,
        &PlatformerAnimator,
        &Velocity,
        &DirectionState,
        &ActionState
    ),
    Changed<ActionState>
>) {
    log::debug!("(SYSTEM) control_animations");
    for (
        mut animation_set,
        animator,
        velocity,
        dir_holder,
        action_state
    ) in platformer_entities.iter_mut() {
        match action_state.0 {
            State::Idle => {
                animation_set.set_grouped_animation(
                    animator.idle_handle,
                    dir_holder.get_direction_index(animator.direction_type),
                    false
                ).unwrap();
            },
            State::Running => {
                let vxz = velocity.0.length_squared();
                animation_set.set_speed(vxz / 4.0);
                animation_set.set_grouped_animation(
                    animator.run_handle,
                    dir_holder.get_direction_index(animator.direction_type),
                    false
                ).unwrap();
            },
            State::Jumping => {
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