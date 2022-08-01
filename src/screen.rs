use bevy::prelude::*;
use bevy::reflect::{Uuid, TypeUuidDynamic};


/// Plugin that defines what a screen is, a registry of screens that can be referred to by name, and logic for loading/unloading said screens.
pub struct ScreenPlugin;
impl Plugin for ScreenPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<CurrentScreen>()
            .add_event::<LoadScreenEvent>()
            .add_event::<ScreenLoadedEvent>()
            .add_system_to_stage(CoreStage::PostUpdate, handle_screen_events);
    }
}

/// Defines a marker trait for screen types.
pub trait ScreenType: TypeUuidDynamic {}
impl<T: TypeUuidDynamic> ScreenType for T {}

/// Name and type of a screen.
#[derive(Clone, Default, Debug)]
pub struct ScreenInfo {
    name: String,
    typ: Uuid
}
impl ScreenInfo {
    pub fn new(name: impl Into<String>, typ: impl ScreenType) -> Self {
        Self {
            name: name.into(),
            typ: typ.type_uuid()
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn is_screen_type(&self, screen_type: impl ScreenType) -> bool {
        self.typ == screen_type.type_uuid()
    }
}

/// Fired to instruct the game to go to a different screen.
#[derive(Clone, Debug)]
pub struct LoadScreenEvent(pub ScreenInfo);
impl LoadScreenEvent {
    pub fn new(name: impl Into<String>, typ: impl ScreenType) -> Self {
        Self(ScreenInfo::new(name, typ))
    }
}

/// Fired when screen finishes loading
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ScreenLoadedEvent;

/// Resource representing the current screen occupied
#[derive(Default)]
pub struct CurrentScreen(pub ScreenInfo);

/// Maker component that tells the engine to never despawn this entity.
/// Useful for UI layer entities.
#[derive(Component)]
pub struct Keep;


/// When a LoadScreenEvent is fired,
/// Set the CurrentScreen resource and delete all entities without the [`Keep`] component.
fn handle_screen_events(
    mut commands: Commands,
    mut load_screen_events: EventReader<LoadScreenEvent>,
    all_entities: Query<Entity, Without<Keep>>,
) {
    // Gets event or quits
    let load_event = match load_screen_events.iter().next() {
        Some(event) => event,
        None => return
    };

    // Despawns entities
    for entity in &all_entities {
        commands.entity(entity).despawn_recursive();
    }

    // Sets current screen resource
    commands.insert_resource(CurrentScreen(load_event.0.clone()));
}