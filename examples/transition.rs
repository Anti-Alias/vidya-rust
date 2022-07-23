use std::time::Duration;

use bevy::prelude::*;

use vidya_rust::extensions::NodeBundleExt;
use vidya_rust::game::{GamePlugins, GameState};
use vidya_rust::transition::FadeTransition;
use vidya_rust::ui::UiLayers;
use vidya_rust::ui_event::{UiEventPlugin, OnClick};

/// Events that can be fired by the title screen
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum TransitionEvent {
    BlackTransition,
    BlueTransition,
}

fn main() {
    App::new()
        .add_plugins(GamePlugins)
        .add_plugin(UiEventPlugin::<TransitionEvent>::default())
        .add_system_set(SystemSet::on_enter(GameState::GameRunning)
            .with_system(create_screen)
        )
        .add_system(handle_events)
        .run();
}

fn create_screen(
    mut commands: Commands,
    layers: Res<UiLayers>,
    asset_server: Res<AssetServer>
) {

    // Loads asset(s)
    let font: Handle<Font> = asset_server.load("fonts/yoster.ttf");

    // Screen container
    let container = commands.spawn_bundle(NodeBundle::vbox(JustifyContent::Center)).with_children(|container| {

        // Transition buttons
        container.spawn_bundle(NodeBundle::packed_hbox()).with_children(|buttons| {

            // Black
            buttons
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    color: Color::NONE.into(),
                    ..default()
                })
                .with_children(|quit_button| {
                    quit_button.spawn_bundle(TextBundle {
                        text: Text::from_section(
                            "Black Transition",
                            TextStyle {
                                font: font.clone(),
                                font_size: 24.0,
                                color: Color::BLACK
                            }
                        ),
                        ..default()
                    });
                })
                .insert(OnClick(TransitionEvent::BlackTransition));


            // Blue
            buttons
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    color: Color::NONE.into(),
                    ..default()
                })
                .with_children(|quit_button| {
                    quit_button.spawn_bundle(TextBundle {
                        text: Text::from_section(
                            "Blue Transition",
                            TextStyle {
                                font: font.clone(),
                                font_size: 24.0,
                                color: Color::BLUE
                            }
                        ),
                        ..default()
                    });
                })
                .insert(OnClick(TransitionEvent::BlueTransition));
        });
    })
    .id();

    // Adds UI container to the ui layer
    commands.entity(layers.ui_layer).add_child(container);
}

fn handle_events(
    mut commands: Commands,
    mut reader: EventReader<TransitionEvent>
) {
    for event in reader.iter() {
        let color = match event {
            TransitionEvent::BlueTransition => Color::BLUE,
            TransitionEvent::BlackTransition => Color::BLACK
        };
        commands.insert_resource(FadeTransition::new(color, Duration::from_secs(2)));
    }
}