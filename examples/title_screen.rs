use bevy::prelude::*;

use vidya_rust::game::GamePlugins;
use vidya_rust::ui::UiLayers;

fn main() {
    App::new()
        .add_plugins(GamePlugins)
        .add_startup_system(create_title_screen)
        .run();
}

fn create_title_screen(
    mut commands: Commands,
    layers: Res<UiLayers>,
    asset_server: Res<AssetServer>
) {
    // Creates screen-size root container, storing children as columns
    let root = commands.spawn_bundle(NodeBundle {
        style: Style {
            flex_direction: FlexDirection::ColumnReverse,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            ..default()
        },
        color: Color::RED.into(),
        ..default()
    }).id();
    //commands.entity(layers.ui_layer).add_child(root);

    // Adds title text to root
    let font: Handle<Font> = asset_server.load("fonts/yoster.ttf");
    let title_text = commands.spawn_bundle(TextBundle {
        text: Text::with_section(
            "Title Screen",
            TextStyle { font, font_size: 30.0, color: Color::WHITE },
            Default::default()
        ),
        ..default()
    }).id();
    commands.entity(root).add_child(title_text);
}