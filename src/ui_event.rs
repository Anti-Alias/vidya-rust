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
        app.add_system(handle_interactions::<E>);
    }
}

/// An event that lies dormant in a Ui entity only to be fired
/// when interacted with.
#[derive(Component)]
pub struct Dormant<E: Event + Clone>(pub E);

/// Handles UI interactions
pub fn handle_interactions<E: Event + Clone>(
    mut writer: EventWriter<E>,
    interacted_nodes: Query<
        (&Interaction, &Dormant<E>),
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