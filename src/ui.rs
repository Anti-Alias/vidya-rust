use bevy::{prelude::*, ui::FocusPolicy};
use crate::util::Permanent;

/// Plugin that adds UI related features for the game
pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_ui_layers);
    }
}
 
/// Keeps track of node layers where UI entities may be placed.
pub struct UiLayers {
    // Screen-sized root layer that contains all other layers
    pub root_layer: Entity,
    // Screen-sized layer that contains all nodes associated with transitions between screens.
    // Useful for loading screens, fade-to-black transitions, etc.
    pub transition_layer: Entity,
    // Screen-sized UI layer
    pub ui_layer: Entity
}

// Sets up UI layers
fn setup_ui_layers(mut commands: Commands) {

    // UI Camera
    commands
        .spawn_bundle(Camera2dBundle {
            camera: Camera {
                priority: 1,
                ..default()
            },
            ..default()
        })
        .insert(Permanent);

    // Root layer
    let root_layer = commands.spawn_bundle(NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            position_type: PositionType::Absolute,
            ..default()
        },
        color: Color::NONE.into(),
        focus_policy: FocusPolicy::Pass,
        ..default()
    }).id();
    
    // Main UI layer
    let ui_layer = commands.spawn_bundle(NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            ..default()
        },
        focus_policy: FocusPolicy::Pass,
        color: Color::NONE.into(),
        ..default()
    })
    .insert(Permanent)
    .id();

    // Layer for transitioning the screen (loading, fade to black, etc)
    let transition_layer = commands.spawn_bundle(NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            justify_content: JustifyContent::Center,
            position_type: PositionType::Absolute,
            ..default()
        },
        focus_policy: FocusPolicy::Pass,
        color: Color::NONE.into(),
        ..default()
    })
    .insert(Permanent)
    .id();

    // Adds sub layers to root layer
    commands.entity(root_layer).add_child(ui_layer).add_child(transition_layer);
    
    // Inserts layer resource
    commands.insert_resource(UiLayers {
        root_layer,
        ui_layer,
        transition_layer
    });
}