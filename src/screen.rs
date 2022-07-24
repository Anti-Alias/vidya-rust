use std::any::{Any, TypeId};

use bevy::{prelude::*, reflect::{TypeUuidDynamic, Uuid}};
use dyn_clone::DynClone;

/// Plugin that defines what a screen is, a registry of screens that can be referred to by name, and logic for loading/unloading said screens.
pub struct ScreenPlugin;
impl Plugin for ScreenPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<CurrentScreen>()
            .add_event::<LoadScreenEvent>()
            .add_system_to_stage(CoreStage::PostUpdate, handle_screen_events);
    }
}

pub trait ScreenType: Any + Send + Sync + DynClone + 'static {
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}
dyn_clone::clone_trait_object!(ScreenType);

/// Name and type of a screen.
#[derive(Clone, Default)]
pub struct ScreenInfo {
    name: String,
    typ: Uuid
}
impl ScreenInfo {

    pub fn new(name: impl Into<String>, typ: impl TypeUuidDynamic) -> Self {
        Self {
            name: name.into(),
            typ: typ.type_uuid()
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_screen_type(&self, screen_type: impl TypeUuidDynamic) -> bool {
        self.typ == screen_type.type_uuid()
    }
}

/// Instructs the game to go to a different screen.
pub struct LoadScreenEvent(pub ScreenInfo);
impl LoadScreenEvent {
    pub fn new(name: impl Into<String>, typ: impl TypeUuidDynamic) -> Self {
        Self(ScreenInfo::new(name, typ))
    }
}

/// Resource representing the current screen occupied
#[derive(Default)]
pub struct CurrentScreen(pub ScreenInfo);


fn handle_screen_events(
    mut commands: Commands,
    mut load_screen_events: EventReader<LoadScreenEvent>,
) {
    let load_event = match load_screen_events.iter().next() {
        Some(event) => event,
        None => return
    };
    commands.insert_resource(CurrentScreen(load_event.0.clone()));
}