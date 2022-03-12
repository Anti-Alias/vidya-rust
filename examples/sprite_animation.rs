use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use vidya_rust::app::VidyaCorePlugin;
use vidya_rust::sprite::{SpritePlugin, Sprite3D, Region, Sprite3DBundle};
use vidya_rust::map::AppState;

fn main() {
    App::new()
        .add_plugin(VidyaCorePlugin)
        .add_plugin(SpritePlugin)
        .add_system_set(SystemSet::on_exit(AppState::AppStarting)
            .with_system(spawn_sprite)
        )
        .run();
}

fn spawn_sprite(
    assets: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands
) {

    // Loads image
    let image_handle = assets.load("player/char_a_p1_0bas_demn_v01.png");

    // Creates from image
    log::info!("Mat count: {}", materials.len());
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
        size: Vec2::new(64.0, 64.0),
        region: Region {
            min: Vec2::new(0.0/8.0, 0.0/8.0),
            max: Vec2::new(1.0/8.0, 1.0/8.0)
        }
    };

    // Spawns entity from bundle
    commands
        .spawn_bundle(Sprite3DBundle {
            sprite,
            material: materials.add(material),
            transform: Transform::from_xyz(200.0, 0.0, 0.0),
            global_transform: GlobalTransform::default()
        });

    // Spawns camera
    let mut camera = OrthographicCameraBundle::new_3d();
    camera.transform = Transform::from_xyz(0.0, 0.0, 100.0)
        .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y);
    camera.orthographic_projection.scaling_mode = ScalingMode::WindowSize;
    commands.spawn_bundle(camera);
}
