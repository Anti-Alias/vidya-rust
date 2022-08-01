use bevy::prelude::*;
use bevy::reflect::TypeUuid;

use super::{TransitionState, TransitionInfo};


/// Plugin that adds fade transitions
pub struct FadeTransitionPlugin;
impl Plugin for FadeTransitionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(SystemSet::on_enter(TransitionState::FirstHalf)
                .with_system(start)
            )
            .add_system_set(SystemSet::on_update(TransitionState::FirstHalf)
                .with_system(first_half)
            )
            .add_system_set(SystemSet::on_update(TransitionState::SecondHalf)
                .with_system(second_half)
            );
    }
}

/// Fade screen type
#[derive(Debug, TypeUuid)]
#[uuid = "02e7cfca-cf09-4511-9415-8b0e1c9dcd6c"]
pub struct FadeTransitionType;

/// Resource that stores information about the fade transition
struct FadeTransitionData {
    entity: Entity
}

// Listens for transition event to kick off transition
fn start(mut commands: Commands, info: Res<TransitionInfo>) {
    
    // Quit if wrong transition type
    if !info.is_type(FadeTransitionType) { return }

    // Spawn screen-sized node
    let node = commands.spawn_bundle(NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            ..default()
        },
        focus_policy: bevy::ui::FocusPolicy::Pass,
        color: Color::BLACK.into(),
        ..default()
    }).id();

    // Inserting resource to track node
    commands.insert_resource(FadeTransitionData {
        entity: node
    });
}

fn first_half(info: Res<TransitionInfo>) {

}

// fn first_half(
//     transition: Option<ResMut<FadeTransition>>,
//     ui_layers: Res<UiLayers>,
//     time: Res<Time>,
//     mut color_query: Query<&mut UiColor>,
//     mut commands: Commands
// ) {
//     log::debug!("(SYSTEM) handle_transition");

//     // Skips if there is no transition resource
//     let mut transition = match transition {
//         Some(transition) => transition,
//         None => return
//     };

//     // Updates timer and calculates alpha
//     transition.timer.tick(time.delta());
//     let alpha = (0.5 - transition.timer.percent()).abs();
//     let alpha = 1.0 - alpha * 2.0;
//     let tcolor = transition.color;

//     // Either spawns transition node, or updates existing one
//     match transition.node {
//         Some(node) => {
//             if transition.timer.finished() {
//                 commands.remove_resource::<FadeTransition>();
//                 commands.entity(ui_layers.transition_layer).despawn_descendants();
//                 return
//             }
//             let node_color = &mut color_query.get_mut(node).unwrap().0;
//             *node_color = Color::rgba(tcolor[0], tcolor[1], tcolor[2], alpha);
//             node
//         },
//         None => {
//             if transition.timer.finished() {
//                 commands.remove_resource::<FadeTransition>();
//                 return
//             }
//             let color = transition.color;
//             let node = commands.spawn_bundle(NodeBundle {
//                 style: Style {
//                     size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
//                     ..default()
//                 },
//                 focus_policy: bevy::ui::FocusPolicy::Pass,
//                 color: Color::rgba(color[0], color[1], color[2], 0.0).into(),
//                 ..default()
//             }).id();
//             commands.entity(ui_layers.transition_layer).add_child(node);
//             transition.node = Some(node);
//             node
//         }
//     };
// }