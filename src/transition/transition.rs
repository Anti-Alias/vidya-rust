use std::{time::Duration, marker::PhantomData};

use bevy::prelude::*;
use bevy::ecs::event::Event;

use crate::{game::{GameState, run_if_tick_elapsed}, ui::UiLayers};

use super::TransitionState;

/// Plugin that allows the user to fade the screen to an opaque color, fire an event,
/// wait for an event response, then fade back.
/// Useful for scene transitions.
pub struct FadeTransitionPlugin<E1: Event, E2: Event> {
    event: Option<E1>,
    marker: PhantomData<E2>
}

impl<E1: Event, E2: Event> Plugin for FadeTransitionPlugin<E1, E2> {
    fn build(&self, app: &mut App) {
        app
            .add_state(TransitionState::Idle)
            .init_resource::<TransitionState>()
            .add_system_set_to_stage(CoreStage::PostUpdate, SystemSet::on_update(GameState::GameRunning)
                .with_run_criteria(run_if_tick_elapsed)
                .with_system(handle_transition)
            );
    }
}


/// Short lived resource that fades to an opaque color before firing an event.
pub struct FadeTransition<E1: Event> {
    color: [f32; 3],
    timer: Timer,
    node: Option<Entity>,
    event_to_fire: Option<E1>
}

impl<E1: Event> FadeTransition<E1> {

    /// New transition. Alpha is ignored if supplied.
    pub fn new(color: Color, duration: Duration, event_to_fire: E1) -> Self {
        if duration <= Duration::ZERO {
            panic!("Invalid duration");
        }
        let color = color.as_rgba_f32();
        Self {
            color: [color[0], color[1], color[2]],
            timer: Timer::new(duration, false),
            node: None,
            event_to_fire: Some(event_to_fire)
        }
    }
}

fn handle_transition<E1: Event, E2: Event>(
    transition: Option<ResMut<FadeTransition>>,
    ui_layers: Res<UiLayers>,
    time: Res<Time>,
    writer: ResMut<EventWriter<E1>>,
    reader: ResMut<EventReader<E2>>,
    mut color_query: Query<&mut UiColor>,
    mut commands: Commands
) {
    log::debug!("(SYSTEM) handle_transition");

    // Skips if there is no transition resource
    let mut transition = match transition {
        Some(transition) => transition,
        None => return
    };

    // Updates timer and calculates alpha
    transition.timer.tick(time.delta());
    let alpha = (0.5 - transition.timer.percent()).abs();
    let alpha = 1.0 - alpha * 2.0;
    let tcolor = transition.color;

    // Either spawns transition node, or updates existing one
    match transition.node {
        Some(node) => {
            if transition.timer.finished() {
                commands.remove_resource::<FadeTransition>();
                commands.entity(ui_layers.transition_layer).despawn_descendants();
                return
            }
            let node_color = &mut color_query.get_mut(node).unwrap().0;
            *node_color = Color::rgba(tcolor[0], tcolor[1], tcolor[2], alpha);
            node
        },
        None => {
            if transition.timer.finished() {
                commands.remove_resource::<FadeTransition>();
                return
            }
            let color = transition.color;
            let node = commands.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    ..default()
                },
                focus_policy: bevy::ui::FocusPolicy::Pass,
                color: Color::rgba(color[0], color[1], color[2], 0.0).into(),
                ..default()
            }).id();
            commands.entity(ui_layers.transition_layer).add_child(node);
            transition.node = Some(node);
            node
        }
    };
}