use bevy::prelude::*;
use vidya_rust::game::{ GameState, GamePlugins };
use vidya_rust::screen::LoadScreenEvent;
use vidya_rust::map::MapScreenType;

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

fn load_map(mut emitter: EventWriter<LoadScreenEvent>) {
    
    // Starts the app
    log::debug!("Entered system 'load_map'");
    emitter.send(LoadScreenEvent::new("maps/tmx/map.tmx", MapScreenType));
    log::debug!("Sent LoadMapEvent event");
}