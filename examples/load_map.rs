use bevy::prelude::*;
use bevy::window::{ WindowMode, WindowResizeConstraints };
use vidya_rust::app::{ AppState, VidyaPlugins };
use vidya_rust::map::{ LoadMapEvent };

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "vidya".to_string(),
            width: 800.0,
            height: 450.0,
            position: None,
            resize_constraints: WindowResizeConstraints::default(),
            scale_factor_override: None,
            vsync: true,
            resizable: true,
            decorations: true,
            cursor_locked: false,
            cursor_visible: true,
            mode: WindowMode::Windowed,
            transparent: false,
        })
        .add_plugins(VidyaPlugins)
        .add_system_set(SystemSet::on_enter(AppState::AppRunning).with_system(load_map))
        .run();
}

fn load_map(mut emitter: EventWriter<LoadMapEvent>) {
    
    // Starts the app
    log::debug!("Entered system 'load_map'");
    emitter.send(LoadMapEvent("maps/tmx/map.tmx".to_string()));
    log::debug!("Sent LoadMapEvent event");
}