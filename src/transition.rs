use crate::{screen::{ScreenInfo, LoadScreenEvent, ScreenLoadedEvent, ScreenType, Keep}, ui::UiLayers};

use bevy::{prelude::*, ui::FocusPolicy};
use std::time::Duration;

/// Plugin that allows the user to fade the screen to an opaque color, kick off screen loading, then fade back in when the loading completes.
#[derive(Debug)]
pub struct TransitionPlugin;
impl Plugin for TransitionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<TransitionEvent>()
            .add_state_to_stage(CoreStage::PreUpdate, TransitionState::Idle)
            .add_system_set_to_stage(CoreStage::PreUpdate, SystemSet::on_update(TransitionState::Idle)
                .with_system(start)
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


/// Event that kicks off a transition.
#[derive(Clone, Debug)]
pub struct TransitionEvent {
    /// Information about the screen to transition to
    pub screen_info: ScreenInfo,
    /// Duration of the fade in and fade out phases of the transition
    pub fade_duration: Duration
}
impl TransitionEvent {

    pub fn new(screen_name: impl Into<String>, screen_type: impl ScreenType) -> Self {
        Self {
            screen_info: ScreenInfo::new(screen_name, screen_type),
            fade_duration: Duration::from_secs(1),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum TransitionState {
    Idle,
    FirstHalf,
    Waiting,
    SecondHalf
}

/// Resoruce that tracks the progress of the transition
pub struct TransitionInfo {
    node: Entity,
    timer: Timer,
    event_to_fire: Option<LoadScreenEvent>
}

fn start(
    mut trans_events: EventReader<TransitionEvent>,
    ui_layers: Res<UiLayers>,
    mut trans_state: ResMut<State<TransitionState>>,
    mut commands: Commands
) {
    // Gets event
    let event = match trans_events.iter().next() {
        Some(event) => event,
        None => return
    };

    // Spawns fullscreen node as a child of the transition layer
    let node = commands.spawn_bundle(NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            ..default()
        },
        color: Color::NONE.into(),
        focus_policy: FocusPolicy::Pass,
        ..default()
    })
    .insert(Keep)
    .id();
    commands.entity(ui_layers.transition_layer).add_child(node);

    // Inserts transition resource
    commands.insert_resource(TransitionInfo {
        node,
        timer: Timer::new(event.fade_duration, false),
        event_to_fire: Some(LoadScreenEvent(event.screen_info.clone()))
    });

    // Sets state to FirstHalf of transition
    trans_state.set(TransitionState::FirstHalf).unwrap();
}

/// Update logic for the first half
fn first_half(
    mut trans_state: ResMut<State<TransitionState>>,
    time: Res<Time>,
    mut transition_info: ResMut<TransitionInfo>,
    mut node_query: Query<&mut UiColor>,
    mut writer: EventWriter<LoadScreenEvent>
) {
    // Updates timer
    let node = transition_info.node;
    let timer = &mut transition_info.timer;
    timer.tick(time.delta());

    // Sets transition node's alpha
    let mut node_color = node_query.get_mut(node).unwrap();
    *node_color = Color::rgba(0.0, 0.0, 0.0, timer.percent()).into();

    // Sends LoadScreenEvent if timer finished
    if timer.finished() {
        timer.reset();
        trans_state.set(TransitionState::Waiting).unwrap();
        writer.send(transition_info.event_to_fire.take().unwrap());
    }
}

/// Logic that waits for ScreenLoadedEvent before continuing to second half
fn waiting(
    mut reader: EventReader<ScreenLoadedEvent>,
    mut trans_state: ResMut<State<TransitionState>>
) {
    if reader.iter().next().is_some() {
        trans_state.set(TransitionState::SecondHalf).unwrap();
        println!("Finished waiting!");
    }
}

/// Update logic for second half
fn second_half(
    mut commands: Commands,
    mut trans_state: ResMut<State<TransitionState>>,
    ui_layers: Res<UiLayers>,
    time: Res<Time>,
    mut transition_info: ResMut<TransitionInfo>,
    mut node_query: Query<&mut UiColor>
) {
    // Gets node / updates timer
    let node = transition_info.node;
    let timer = &mut transition_info.timer;
    timer.tick(time.delta());

    // Sets transition node's alpha
    let mut node_color = node_query.get_mut(node).unwrap();
    *node_color = Color::rgba(0.0, 0.0, 0.0, timer.percent_left()).into();

    // If timer finished, remove node/resource, and go back to idle
    if timer.finished() {
        timer.reset();
        trans_state.set(TransitionState::Idle).unwrap();
        commands.remove_resource::<TransitionInfo>();
        commands.entity(ui_layers.transition_layer).despawn_descendants();
        println!("Finished!");
    }
}