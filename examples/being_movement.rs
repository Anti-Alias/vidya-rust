use std::time::Duration;

use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

use vidya_rust::extensions::*;
use vidya_rust::animation::{SpriteAnimationBundle, AnimationSet, Animation, AnimationTimer, AnimationPlugin, AnimationHandle};
use vidya_rust::app::VidyaCorePlugin;
use vidya_rust::being::BeingPlugin;
use vidya_rust::sprite::SpritePlugin;
use vidya_rust::map::AppState;

fn main() {
    App::new()
        .add_plugin(VidyaCorePlugin)
        .add_plugin(SpritePlugin)
        .add_plugin(AnimationPlugin)
        .add_plugin(BeingPlugin)
        .add_system_set(SystemSet::on_exit(AppState::AppStarting)
            .with_system(spawn_being)
        )
        .run();
}

fn spawn_being(
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
        let idle_handle = animation_set.add_animation_group(&[idle_e, idle_n, idle_w, idle_s]);
        let walk_handle = animation_set.add_animation_group(&[walk_e, walk_n, walk_w, walk_s]);
    
        // Spawns entity from bundle
        commands
            .spawn_bundle(SpriteAnimationBundle::new(
                animation_set,
                AnimationTimer(Timer::new(Duration::from_millis(1000/15), true)),
                materials.add(player_mat),
                Transform::from_xyz(-128.0, -128.0, 0.0).with_scale(Vec3::new(4.0, 4.0, 1.0)),
                GlobalTransform::default()
            ));
    
        // Spawns camera
        let mut camera = OrthographicCameraBundle::new_3d();
        camera.transform = Transform::from_xyz(0.0, 0.0, 100.0)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y);
        camera.orthographic_projection.scaling_mode = ScalingMode::WindowSize;
        commands.spawn_bundle(camera);
}