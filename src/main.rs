use bevy::prelude::*;
use bevy::window::{ WindowMode, WindowResizeConstraints };
use vidya_rust::app::{ AppState, VidyaPlugin };
use vidya_rust::map::{ LoadMapEvent };

/*
fn add_entities(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {

    // Makes mesh and material
    let tex_handle: Handle<Image> = asset_server.load("images/wood.png");
    let quad_handle = meshes.add(Mesh::from(Quad::new(Vec2::new(1.0, 1.0))));
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(tex_handle),
        metallic: 0.0,
        reflectance: 0.0,
        unlit: true,
        ..Default::default()
    });

    // Spawns mesh
    commands.spawn_bundle(PbrBundle {
        mesh: quad_handle,
        material: material_handle,
        transform: Transform::from_scale(Vec3::new(32.0, 32.0, 1.0)),
        ..Default::default()
    });

    // Spawns camera
    let cam_width = 800.0;
    let cam_height = 450.0;
    let mut cam_bundle = OrthographicCameraBundle::new_3d();
    let proj = &mut cam_bundle.orthographic_projection;
    proj.scaling_mode = ScalingMode::None;
    proj.left = -cam_width / 2.0;
    proj.right = cam_width / 2.0;
    proj.bottom = -cam_height / 2.0;
    proj.top = cam_height /2.0;
    proj.near = 0.1;
    proj.far = 1000.0;

    cam_bundle.transform = Transform::from_translation(Vec3::new(0.0, 500.0, 500.0))
        .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0))
        .with_scale(Vec3::new(1.0, 1.0/SQRT_2, 1.0));
    commands.spawn_bundle(cam_bundle);
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
        .add_plugins(DefaultPlugins)
        .add_plugin(VidyaPlugin)
        .add_system_set(SystemSet::on_enter(AppState::AppRunning).with_system(load_map))
        .run();
}

fn load_map(mut emitter: EventWriter<LoadMapEvent>) {
    
    // Starts the app
    log::debug!("Entered system 'load_map'");
    emitter.send(LoadMapEvent("maps/tmx/map.tmx".to_string()));
    log::debug!("Sent LoadMapEvent event");
}