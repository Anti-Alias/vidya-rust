use bevy::prelude::*;

use vidya_rust::extensions::NodeBundleExt;
use vidya_rust::game::GamePlugins;
use vidya_rust::map::MapScreenType;
use vidya_rust::transition::TransitionEvent;
use vidya_rust::ui::UiLayers;
use vidya_rust::ui_event::{UiEventPlugin, OnClick};


fn main() {
    App::new()
        .add_plugins(GamePlugins)
        .add_plugin(UiEventPlugin::<TransitionEvent>::default())
        .add_startup_system(create_screen)
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
    let mut cbundle = NodeBundle::vbox(JustifyContent::Center);
    cbundle.color = Color::GRAY.into();
    let container = commands.spawn_bundle(cbundle).with_children(|container| {

        // Transition buttons
        container.spawn_bundle(NodeBundle::packed_hbox()).with_children(|buttons| {

            // Button
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
                            "Go To Map!",
                            TextStyle {
                                font: font.clone(),
                                font_size: 24.0,
                                color: Color::BLACK
                            }
                        ),
                        ..default()
                    });
                })
                .insert(OnClick(TransitionEvent::fade("maps/tmx/map.tmx", MapScreenType)));
        });
    })
    .id();

    // Adds UI container to the ui layer
    commands.entity(layers.ui_layer).add_child(container);
}