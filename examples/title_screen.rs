use bevy::prelude::*;

use vidya_rust::extensions::NodeBundleExt;
use vidya_rust::game::{GamePlugins, GameState};
use vidya_rust::ui::UiLayers;
use vidya_rust::ui_event::{UiEventPlugin, OnClick};

/// Events that can be fired by the title screen
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum TitleScreenEvent {
    StartGame,
    QuitGame,
    OpenOptions
}

fn main() {
    App::new()
        .add_plugins(GamePlugins)
        .add_plugin(UiEventPlugin::<TitleScreenEvent>::default())
        .add_system_set(SystemSet::on_enter(GameState::GameRunning)
            .with_system(create_title_screen)
        )
        .add_system(handle_events)
        .run();
}

fn create_title_screen(
    mut commands: Commands,
    layers: Res<UiLayers>,
    asset_server: Res<AssetServer>
) {

    // Loads assets
    let bg_image: Handle<Image> = asset_server.load("backgrounds/title.png");
    let font: Handle<Font> = asset_server.load("fonts/yoster.ttf");

    // Background
    commands.spawn_bundle(NodeBundle::cbox()).with_children(|container| {
        container.spawn_bundle(ImageBundle {
            transform: Transform::from_scale(Vec3::new(2.0, 2.0, 1.0)),
            style: Style {
                position_type: PositionType::Absolute,
                ..default()
            },
            image: UiImage(bg_image),
            ..default()
        });
    });

    // Builds UI container
    let ui_container = commands.spawn_bundle(NodeBundle::vbox(JustifyContent::Center)).with_children(|ui_container| {

        // Title
        ui_container.spawn_bundle(TextBundle {
            style: Style {
                margin: UiRect::all(Val::Px(40.0)),
                ..default()
            },
            text: Text::with_section(
                "MMO Maker",
                TextStyle {
                    font: font.clone(),
                    font_size: 48.0,
                    color: Color::WHITE
                },
                Default::default()
            ),
            ..default()
        });

        // Start game / Options / Quit
        ui_container.spawn_bundle(NodeBundle::packed_hbox()).with_children(|buttons| {

            // Start button
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
                        text: Text::with_section(
                            "Start Game",
                            TextStyle {
                                font: font.clone(),
                                font_size: 24.0,
                                color: Color::WHITE
                            },
                            Default::default()
                        ),
                        ..default()
                    });
                })
                .insert(OnClick(TitleScreenEvent::StartGame));


            // Options button
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
                        text: Text::with_section(
                            "Options",
                            TextStyle {
                                font: font.clone(),
                                font_size: 24.0,
                                color: Color::WHITE
                            },
                            Default::default()
                        ),
                        ..default()
                    });
                })
                .insert(OnClick(TitleScreenEvent::OpenOptions));

            // Quit button
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
                        text: Text::with_section(
                            "Quit",
                            TextStyle {
                                font: font.clone(),
                                font_size: 24.0,
                                color: Color::WHITE
                            },
                            Default::default()
                        ),
                        ..default()
                    });
                })
                .insert(OnClick(TitleScreenEvent::QuitGame));
        });
    })
    .id();

    // Adds UI container to the ui layer
    commands.entity(layers.ui_layer).add_child(ui_container);
}

fn handle_events(mut reader: EventReader<TitleScreenEvent>) {
    for event in reader.iter() {
        match event {
            TitleScreenEvent::StartGame => println!("Starting Game!"),
            TitleScreenEvent::OpenOptions => println!("Opening options!"),
            TitleScreenEvent::QuitGame => println!("Quitting Game!")
        }
    }
}