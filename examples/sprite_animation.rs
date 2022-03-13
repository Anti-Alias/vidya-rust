use std::time::Duration;

use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use vidya_rust::animation::{SpriteAnimationBundle, SpriteAnimationSet, SpriteAnimation, AnimationTimer, AnimationPlugin};
use vidya_rust::app::VidyaCorePlugin;
use vidya_rust::sprite::SpritePlugin;
use vidya_rust::map::AppState;

fn main() {
    App::new()
        .add_plugin(VidyaCorePlugin)
        .add_plugin(SpritePlugin)
        .add_plugin(AnimationPlugin)
        .add_system_set(SystemSet::on_exit(AppState::AppStarting)
            .with_system(spawn_sprite_animation)
        )
        .run();
}

fn spawn_sprite_animation(
    assets: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands
) {
        // Loads material from single image
        let image_handle = assets.load("player/char_a_p1_0bas_demn_v01.png");
        let material = StandardMaterial {
            base_color_texture: Some(image_handle),
            metallic: 0.0,
            reflectance: 0.0,
            perceptual_roughness: 1.0,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..Default::default()
        };
    
        // Gets walk animation
        let walk_anim = SpriteAnimation::from_grid(0, 64*4, 64, 64, 512, 512, 6);
        let mut animation_set = SpriteAnimationSet::new();
        let walk_anim_handle = animation_set.add_animation(walk_anim);
        animation_set.set_animation(walk_anim_handle).unwrap();
    
        // Spawns entity from bundle
        commands
            .spawn_bundle(SpriteAnimationBundle::new(
                animation_set,
                AnimationTimer(Timer::new(Duration::from_millis(1000/15), true)),
                materials.add(material),
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