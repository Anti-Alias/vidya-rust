use bevy::prelude::*;

use vidya_rust::{game::GamePlugins, util::Permanent};
use vidya_rust::ui::UiLayers;

fn main() {
    App::new()
        .add_plugins(GamePlugins)
        .add_startup_system(create_title_screen)
        .run();
}

fn create_title_screen(mut commands: Commands, layers: Res<UiLayers>) {
    
}