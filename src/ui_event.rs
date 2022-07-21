use std::marker::PhantomData;

use bevy::{prelude::*, ecs::event::Event};

/// Plugin that allows the firing of events of a certain type when
/// Ui entities are interacted with.
pub struct UiEventPlugin<E: Event + Clone> { marker: PhantomData<E> }
impl<E: Event + Clone> Default for UiEventPlugin<E> {
    fn default() -> Self {
        Self {
            marker: PhantomData::default()
        }
    }
}

impl<E: Event + Clone> Plugin for UiEventPlugin<E> {
    fn build(&self, app: &mut App) {
        app.add_event::<E>();
        app.add_system(handle_clicks::<E>);
        app.add_system(handle_hovers::<E>);
    }
}

/// A Component that stores an event to be fired when the entity is clicked
#[derive(Component)]
pub struct OnClick<E: Event + Clone>(pub E);

/// A Component that stores an event to be fired when the entity is hovered
#[derive(Component)]
pub struct OnHover<E: Event + Clone>(pub E);

/// Handles click interactions
fn handle_clicks<E: Event + Clone>(
    mut writer: EventWriter<E>,
    interacted_nodes: Query<
        (&Interaction, &OnClick<E>),
        Changed<Interaction>
    >
) {
    for (interaction, action) in interacted_nodes.iter() {
        match interaction {
            Interaction::Clicked => {
                writer.send(action.0.clone())
            },
            _ => {}
        }
    }
}

/// Handles hover interactions
fn handle_hovers<E: Event + Clone>(
    mut writer: EventWriter<E>,
    interacted_nodes: Query<
        (&Interaction, &OnHover<E>),
        Changed<Interaction>
    >
) {
    for (interaction, action) in interacted_nodes.iter() {
        match interaction {
            Interaction::Hovered => {
                writer.send(action.0.clone())
            },
            _ => {}
        }
    }
}