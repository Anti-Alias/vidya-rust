use std::time::Duration;

use bevy::prelude::*;

use crate::{game::{GameState, run_if_tick_elapsed}, ui::UiLayers};

pub struct FadeTransitionPlugin;
impl Plugin for FadeTransitionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(SystemSet::on_update(GameState::GameRunning)
                .with_run_criteria(run_if_tick_elapsed)
                .with_system(handle_transition)
            );
    }
}

/// Short lived resource that fades to an opaque color before switching screens.
pub struct FadeTransition {
    color: [f32; 3],
    timer: Timer,
    node: Option<Entity>
}

impl FadeTransition {

    /// New transition. Alpha is ignored if supplied.
    pub fn new(color: Color, duration: Duration) -> Self {
        if duration <= Duration::ZERO {
            panic!("Invalid duration");
        }
        let color = color.as_rgba_f32();
        Self {
            color: [color[0], color[1], color[2]],
            timer: Timer::new(duration, false),
            node: None
        }
    }
}

fn handle_transition(
    mut commands: Commands,
    ui_layers: Res<UiLayers>,
    transition: Option<ResMut<FadeTransition>>,
    time: Res<Time>,
    mut color_query: Query<&mut UiColor>
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
                println!("Done!");
                commands.entity(node).despawn();
                return
            }
            println!("Changing color");
            // let node_color = &mut color_query.get_mut(node).unwrap().0;
            // *node_color = Color::rgba(tcolor[0], tcolor[1], tcolor[2], alpha);
            node
        },
        None => {
            if transition.timer.finished() {
                println!("Before");
                commands.remove_resource::<FadeTransition>();
                println!("After");
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