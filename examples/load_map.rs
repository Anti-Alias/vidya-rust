use bevy::prelude::*;
use vidya_rust::game::{ GameState, GamePlugins };
use vidya_rust::map::{ LoadMapEvent };

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "vidya".to_string(),
            width: 800.0,
            height: 450.0,
            ..Default::default()
        })
        .add_plugins(GamePlugins)
        .add_system_set(SystemSet::on_enter(GameState::GameRunning).with_system(load_map))
        .run();
}

fn load_map(mut emitter: EventWriter<LoadMapEvent>) {
    
    // Starts the app
    log::debug!("Entered system 'load_map'");
    emitter.send(LoadMapEvent("maps/tmx/map.tmx".to_string()));
    log::debug!("Sent LoadMapEvent event");
}