use std::time::Duration;

use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

use bevy::window::{WindowResizeConstraints, WindowMode};
use vidya_rust::extensions::*;
use vidya_rust::animation::{SpriteAnimationBundle, AnimationSet, Animation, AnimationTimer, AnimationPlugin};
use vidya_rust::app::{VidyaCorePlugin, VidyaPlugins};
use vidya_rust::map::LoadMapEvent;
use vidya_rust::platformer::{PlatformerPlugin, Platformer};
use vidya_rust::being::Being;
use vidya_rust::physics::{PhysicsPlugin, Velocity, Friction, Position};
use vidya_rust::player::{PlayerPlugin, Player};
use vidya_rust::sprite::SpritePlugin;
use vidya_rust::app::AppState;

/*
fn main() {
    App::new()
        .add_plugins(VidyaPlugins)
        .add_system_set(SystemSet::on_exit(AppState::AppStarting)
            .with_system(spawn_player)
        )
        .run();
}
*/

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
        .add_system_set(SystemSet::on_enter(AppState::AppRunning)
            .with_system(load_map)
            .with_system(spawn_player)
        )
        .run();
}

fn load_map(mut emitter: EventWriter<LoadMapEvent>) {
    
    // Starts the app
    log::debug!("Entered system 'load_map'");
    emitter.send(LoadMapEvent("maps/tmx/map.tmx".to_string()));
    log::debug!("Sent LoadMapEvent event");
}

fn spawn_player(
    assets: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands
) {
        // Loads material from single image
        let player_mat = StandardMaterial::from_image("player/char_a_p1_0bas_demn_v01.png", AlphaMode::Blend, &assets);
    
        // Creates animation set
        let mut animation_set = AnimationSet::new();
        let idle_n = Animation::from_grid(0, 1*64, 64, 64, 512, 512, 1);
        let idle_s = Animation::from_grid(0, 0*64, 64, 64, 512, 512, 1);
        let idle_e = Animation::from_grid(0, 2*64, 64, 64, 512, 512, 1);
        let idle_w = Animation::from_grid(0, 3*64, 64, 64, 512, 512, 1);
        let walk_n = Animation::from_grid(0, 5*64, 64, 64, 512, 512, 6);
        let walk_s = Animation::from_grid(0, 4*64, 64, 64, 512, 512, 6);
        let walk_e = Animation::from_grid(0, 6*64, 64, 64, 512, 512, 6);
        let walk_w = Animation::from_grid(0, 7*64, 64, 64, 512, 512, 6);
        let _idle_handle = animation_set.add_animation_group(&[idle_e, idle_n, idle_w, idle_s]);
        let _walk_handle = animation_set.add_animation_group(&[walk_e, walk_n, walk_w, walk_s]);
    
        // Spawns entity from bundle
        commands
            .spawn()
            .insert_bundle(SpriteAnimationBundle::new(
                animation_set,
                AnimationTimer(Timer::new(Duration::from_millis(1000/15), true)),
                materials.add(player_mat),
                Transform::default(),
                GlobalTransform::default()
            ))
            .insert(Position(Vec3::new(0.0, 0.0, 0.0)))
            .insert(Velocity::default())
            .insert(Friction {
                xz: 0.7,
                y: 1.0
            })
            .insert(Player)
            .insert(Being::default())
            .insert(Platformer::new(10.0))
        ;
}