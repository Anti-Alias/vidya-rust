use std::time::Duration;

use vidya_rust::camera::Targetable;
use vidya_rust::extensions::*;
use vidya_rust::animation::{AnimationSetBundle, AnimationSet, Animation, AnimationTimer};
use vidya_rust::app::VidyaPlugins;
use vidya_rust::map::{LoadMapEvent, MapSpawnedEvent};
use vidya_rust::platformer::{Platformer, PlatformerAnimator};
use vidya_rust::direction::{DirectionHolder, DirectionType};
use vidya_rust::physics::{Friction, Position, SizeCylinder, Weight, PhysicsBundle};
use vidya_rust::player::Player;
use vidya_rust::app::AppState;
use vidya_rust::state::StateHolder;

use bevy::prelude::*;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "vidya".to_string(),
            width: 16.0*80.0,
            height: 9.0*80.0,
            ..Default::default()
        })
        .add_plugins(VidyaPlugins)
        .add_system_set(SystemSet::on_enter(AppState::AppRunning)
            .with_system(load_map)
        )
        .add_system_set(SystemSet::on_update(AppState::AppRunning)
            .with_system(spawn_player)
        )
        .run();
}

// Kicks off map loading
fn load_map(mut emitter: EventWriter<LoadMapEvent>) {
    log::debug!("Entered system 'load_map'");
    emitter.send(LoadMapEvent("maps/tmx/map.tmx".to_string()));
    log::debug!("Sent LoadMapEvent event");
}

// Spawns player after map finishes loading
fn spawn_player(
    assets: Res<AssetServer>,
    mut events: EventReader<MapSpawnedEvent>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands
) {

    // Waits for the map to fully spawn
    match events.iter().next() {
        None => return,
        Some(_) => {}
    }

    // Loads material from single image
    let player_mat = StandardMaterial::from_image("player/char_a_p1_0bas_demn_v01.png", AlphaMode::Mask(0.5), &assets);

    // Creates animation set
    let sprite_offset = Vec3::new(-31.0, -32.0 - 2.0, -10.0);
    let mut animation_set = AnimationSet::new();
    let idle_n = Animation::from_grid(0, 1*64, 64, 64, 512, 512, 1, sprite_offset);
    let idle_s = Animation::from_grid(0, 0*64, 64, 64, 512, 512, 1, sprite_offset);
    let idle_e = Animation::from_grid(0, 2*64, 64, 64, 512, 512, 1, sprite_offset);
    let idle_w = Animation::from_grid(0, 3*64, 64, 64, 512, 512, 1, sprite_offset);
    let run_n = Animation::from_grid(0, 5*64, 64, 64, 512, 512, 6, sprite_offset);
    let run_s = Animation::from_grid(0, 4*64, 64, 64, 512, 512, 6, sprite_offset);
    let run_e = Animation::from_grid(0, 6*64, 64, 64, 512, 512, 6, sprite_offset);
    let run_w = Animation::from_grid(0, 7*64, 64, 64, 512, 512, 6, sprite_offset);
    let jump_n = Animation::from_grid(6*64, 1*64, 64, 64, 512, 512, 1, sprite_offset);
    let jump_s = Animation::from_grid(6*64, 0*64, 64, 64, 512, 512, 1, sprite_offset);
    let jump_e = Animation::from_grid(6*64, 2*64, 64, 64, 512, 512, 1, sprite_offset);
    let jump_w = Animation::from_grid(6*64, 3*64, 64, 64, 512, 512, 1, sprite_offset);
    let idle_handle = animation_set.add_animation_group(&[idle_e, idle_n, idle_w, idle_s]);
    let run_handle = animation_set.add_animation_group(&[run_e, run_n, run_w, run_s]);
    let jump_handle = animation_set.add_animation_group(&[jump_e, jump_n, jump_w, jump_s]);

    // Spawns platformer entity from bundle
    commands
        .spawn()
        .insert_bundle(AnimationSetBundle::new(
            animation_set,
            AnimationTimer::new(Duration::from_millis(1000/15)),
            materials.add(player_mat),
            Transform::default(),
            GlobalTransform::default()
        ))
        .insert_bundle(PhysicsBundle::new(
            Position(Vec3::new(8.0, 100.0, 0.0*16.0)),
            SizeCylinder{
                radius: 6.0,
                half_height: 15.0
            },
            Friction {
                xz: 0.5,
                y: 1.0,
            },
            Weight(0.5)
        ))
        .insert(Player)
        .insert(DirectionHolder::default())
        .insert(StateHolder::default())
        .insert(Platformer::new(2.0))
        .insert(PlatformerAnimator {
            direction_type: DirectionType::FourWay,
            idle_handle,
            run_handle,
            jump_handle
        })
        .insert(Targetable)
    ;
}