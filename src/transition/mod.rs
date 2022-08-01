mod fade;

pub use fade::*;
use crate::screen::{ScreenType, ScreenInfo, LoadScreenEvent, ScreenLoadedEvent};

use bevy::{prelude::*, reflect::{Uuid, TypeUuidDynamic, TypeUuid}};
use std::time::Duration;

/// Plugin that allows the user to fade the screen to an opaque color, kick off screen loading, then fade back in when the loading completes.
#[derive(Debug)]
pub struct TransitionPlugin;
impl Plugin for TransitionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<TransitionEvent>()
            .add_state_to_stage(CoreStage::PreUpdate, TransitionState::Idle)
            .add_plugin(FadeTransitionPlugin)
            .add_system_set_to_stage(CoreStage::PreUpdate, SystemSet::on_update(TransitionState::Idle)
                .with_system(listen_for_transition_event)
            )
            .add_system_set_to_stage(CoreStage::PreUpdate, SystemSet::on_update(TransitionState::FirstHalf)
                .with_system(first_half)
            )
            .add_system_set_to_stage(CoreStage::PreUpdate, SystemSet::on_update(TransitionState::Waiting)
                .with_system(waiting)
            )
            .add_system_set_to_stage(CoreStage::PreUpdate, SystemSet::on_update(TransitionState::SecondHalf)
                .with_system(second_half)
            );
    }
}

/// Represents a type of transition.
pub trait TransitionType: TypeUuidDynamic {}
impl<T: TypeUuidDynamic> TransitionType for T {}

/// Event that kicks off a transition.
#[derive(Clone, Debug)]
pub struct TransitionEvent {
    /// Information about the screen to transition to
    pub screen_info: ScreenInfo,
    /// Transition type
    pub transition_type: Uuid,
    /// Duration of the fade in and fade out phases of the transition
    pub fade_duration: Duration
}
impl TransitionEvent {

    pub fn new(
        screen_info: ScreenInfo,
        transition_type: impl TransitionType,
        fade_duration: Duration
    ) -> Self {
        Self {
            screen_info,
            transition_type: transition_type.type_uuid(),
            fade_duration,
        }
    }

    /// Creates a simple fade transition
    pub fn fade(screen_name: impl Into<String>, screen_type: impl ScreenType) -> Self {
        Self {
            screen_info: ScreenInfo::new(screen_name, screen_type),
            transition_type: FadeTransitionType::TYPE_UUID,
            fade_duration: Duration::from_secs(1)
        }
    }

    pub fn is_type(&self, transition_type: impl TransitionType) -> bool {
        self.transition_type == transition_type.type_uuid()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum TransitionState {
    Idle,
    FirstHalf,
    Waiting,
    SecondHalf
}

/// Keeps track of the progress of a transition
#[derive(Debug, Clone)]
pub struct TransitionInfo {
    timer: Timer,
    event_to_fire: Option<LoadScreenEvent>,
    transition_type: Uuid
}
impl TransitionInfo {
    pub fn percent(&self) -> f32 {
        self.timer.percent()
    }
    pub fn is_type(&self, transition_type: impl TransitionType) -> bool {
        return self.transition_type == transition_type.type_uuid()
    }
}

fn listen_for_transition_event(
    mut trans_events: EventReader<TransitionEvent>,
    mut trans_state: ResMut<State<TransitionState>>,
    mut commands: Commands
) {
    for event in trans_events.iter() {
        commands.insert_resource(TransitionInfo {
            timer: Timer::new(event.fade_duration, false),
            event_to_fire: Some(LoadScreenEvent(event.screen_info.clone())),
            transition_type: event.transition_type
        });
        trans_state.set(TransitionState::FirstHalf).unwrap();
    }
}

fn first_half(
    mut trans_state: ResMut<State<TransitionState>>,
    mut progress: ResMut<TransitionInfo>,
    time: Res<Time>,
    mut writer: EventWriter<LoadScreenEvent>,
) {
    let timer = &mut progress.timer;
    timer.tick(time.delta());
    println!("First half percent: {}", timer.percent());
    if timer.finished() {
        timer.reset();
        trans_state.set(TransitionState::Waiting).unwrap();
        writer.send(progress.event_to_fire.take().unwrap());
        println!("Fired!");
    }
}

fn waiting(
    mut reader: EventReader<ScreenLoadedEvent>,
    mut trans_state: ResMut<State<TransitionState>>
) {
    if reader.iter().next().is_some() {
        trans_state.set(TransitionState::SecondHalf).unwrap();
        println!("Finished waiting!");
    }
}

fn second_half(
    mut commands: Commands,
    mut trans_state: ResMut<State<TransitionState>>,
    mut progress: ResMut<TransitionInfo>,
    time: Res<Time>
) {
    let timer = &mut progress.timer;
    timer.tick(time.delta());
    println!("Second half percent: {}", timer.percent());
    if timer.finished() {
        timer.reset();
        trans_state.set(TransitionState::Idle).unwrap();
        commands.remove_resource::<TransitionInfo>();
        println!("Second half finished!");
    }
}