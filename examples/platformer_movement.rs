use std::time::Duration;

use vidya_rust::camera::Targetable;
use vidya_rust::extensions::*;
use vidya_rust::animation::{AnimationSetBundle, AnimationSet, Animation, AnimationTimer};
use vidya_rust::app::VidyaPlugins;
use vidya_rust::map::LoadMapEvent;
use vidya_rust::platformer::{Platformer, PlatformerAnimator};
use vidya_rust::being::{Being, DirectionType};
use vidya_rust::physics::{Friction, Position, SizeCylinder, Weight, PhysicsBundle};
use vidya_rust::player::Player;
use vidya_rust::app::AppState;
use vidya_rust::state::StateHolder;

use bevy::prelude::*;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "vidya".to_string(),
            width: 1600.0,
            height: 900.0,
            ..Default::default()
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
    emitter.send(LoadMapEvent("maps/tmx/smallmap.tmx".to_string()));
    log::debug!("Sent LoadMapEvent event");
}

fn spawn_player(
    assets: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands
) {
        // Loads material from single image
        let player_mat = StandardMaterial::from_image("player/char_a_p1_0bas_demn_v01_red.png", AlphaMode::Mask(0.5), &assets);
    
        // Creates animation set
        let offset = Vec3::new(-32.0, -21.0, 0.0);
        let mut animation_set = AnimationSet::new();
        let idle_n = Animation::from_grid(0, 1*64, 64, 64, 512, 512, 1, offset);
        let idle_s = Animation::from_grid(0, 0*64, 64, 64, 512, 512, 1, offset);
        let idle_e = Animation::from_grid(0, 2*64, 64, 64, 512, 512, 1, offset);
        let idle_w = Animation::from_grid(0, 3*64, 64, 64, 512, 512, 1, offset);
        let run_n = Animation::from_grid(0, 5*64, 64, 64, 512, 512, 6, offset);
        let run_s = Animation::from_grid(0, 4*64, 64, 64, 512, 512, 6, offset);
        let run_e = Animation::from_grid(0, 6*64, 64, 64, 512, 512, 6, offset);
        let run_w = Animation::from_grid(0, 7*64, 64, 64, 512, 512, 6, offset);
        let jump_n = Animation::from_grid(6*64, 1*64, 64, 64, 512, 512, 1, offset);
        let jump_s = Animation::from_grid(6*64, 0*64, 64, 64, 512, 512, 1, offset);
        let jump_e = Animation::from_grid(6*64, 2*64, 64, 64, 512, 512, 1, offset);
        let jump_w = Animation::from_grid(6*64, 3*64, 64, 64, 512, 512, 1, offset);
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
                Position(Vec3::new(8.0, 40.0, -8.0)),
                SizeCylinder{
                    radius: 64.0,
                    half_height: 32.0
                },
                Friction {
                    xz: 0.5,
                    y: 1.0,
                },
                Weight(0.025)
            ))
            .insert(Player)
            .insert(Being::default())
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