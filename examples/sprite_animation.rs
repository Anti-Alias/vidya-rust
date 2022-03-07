use std::time::Duration;

use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use vidya_rust::app::VidyaCorePlugin;
use vidya_rust::graphics::{GraphicsPlugin, Sprite3D, Rect, Sprite3DBundle};
use vidya_rust::map::AppState;

fn main() {
    App::new()
        .add_plugin(VidyaCorePlugin)
        .add_plugin(GraphicsPlugin)
        .add_system_set(SystemSet::on_exit(AppState::AppStarting).with_system(spawn_sprite))
        .add_system_set(SystemSet::on_update(AppState::AppRunning).with_system(despawn_sprite))
        .run();
}

fn spawn_sprite(
    assets: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands
) {

    // Loads image
    let image_handle = assets.load("player/char_a_p1_0bas_demn_v01.png");
    //let image_handle = assets.load("images/wood.png");

    // Creates from image
    let material = StandardMaterial {
        base_color_texture: Some(image_handle),
        metallic: 0.0,
        reflectance: 0.0,
        perceptual_roughness: 1.0,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..Default::default()
    };

    // Makes sprite that will cut out a slice of the material
    let sprite = Sprite3D {
        size: Vec2::new(512.0, 512.0),
        region: Rect {
            min: Vec2::new(0.0, 0.0),
            max: Vec2::new(1.0, 1.0)
        }
    };

    // Creates resources necessary for the sprite
    commands
        .spawn_bundle(Sprite3DBundle {
            sprite,
            material: materials.add(material),
            transform: Transform::default(),
            global_transform: GlobalTransform::default()
        })
        .insert(Timer::new(Duration::from_secs(3), false))
    ;
    log::info!("Spawning entity...");

    // Spawns camera
    let mut camera = OrthographicCameraBundle::new_3d();
    camera.transform = Transform::from_xyz(0.0, 0.0, 100.0)
        .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y);
    let w = 512.0;
    let h = 512.0;
    camera.orthographic_projection.scaling_mode = ScalingMode::WindowSize;
    commands.spawn_bundle(camera);
}

fn despawn_sprite(
    mut sprite_query: Query<(Entity, &mut Timer), With<Sprite3D>>,
    time: Res<Time>,
    mut commands: Commands
) {
    /*
    for (entity, mut timer) in sprite_query.iter_mut() {
        timer.tick(time.delta());
        if timer.just_finished() {
            commands.entity(entity).despawn();
            log::info!("Despawned entity!");
        }
    }
    */
}