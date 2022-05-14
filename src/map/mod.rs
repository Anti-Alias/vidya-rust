mod events;
mod vidya_map;
mod current_map;
mod current_map_graphics;
mod tile;
mod traverse;

use std::f32::consts::SQRT_2;
use std::iter::Iterator;
use std::path::PathBuf;

use crate::app::AppState;
use crate::camera::{CameraBundle, CameraTargetSettings};
use crate::physics::{ Position, Velocity, Friction, Terrain };
use crate::debug::Floater;
use crate::extensions::*;

use bevy::prelude::*;
use bevy::asset::{ AssetServerSettings, LoadState };
use bevy::render::camera::ScalingMode;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;

pub use current_map::*;
pub use current_map_graphics::*;
pub use events::*;
pub use vidya_map::*;
pub use tile::*;
pub use traverse::*;

pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<LoadMapEvent>()
            .add_asset::<VidyaMap>()
            .init_asset_loader::<VidyaMapLoader>()
            .insert_resource(MapConfig {
                chunk_size: Vec3::new(
                    (16*16) as f32,
                    (16*16) as f32,
                    (16*16) as f32
                ),
                flip_y: false
            })
            // Listens for "LoadMapEvent" and kicks off map loading
            .add_system_set(SystemSet::on_update(AppState::AppRunning)
                .with_system(map_listen)
            )

            // Halts further progress until map is loaded
            .add_system_set(SystemSet::on_update(AppState::MapLoadingFile)
                .with_system(map_finish_loading)
            )

            .add_system_set(SystemSet::on_enter(AppState::MapConstructing)
                .with_system(map_construct)
            )
            .add_system_set(SystemSet::on_update(AppState::MapSpawningEntities)
                .with_system(map_spawn_entities)
            )
        ;
    }
}

// 1) Listens for "LoadMapEvent"
// 2) Begins loading map specified
// 3) Goes to LoadingMap state
fn map_listen(
    mut events: EventReader<LoadMapEvent>,
    mut state: ResMut<State<AppState>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands
) {
    log::debug!("(SYSTEM) on_load_map_event");
    if let Some(event) = events.iter().next() {

        // Begins loading map and stores the handle for later use
        let map_file = &event.0;
        let map_handle = asset_server.load(map_file);

        // Spawns map entity, and inserts CurrentMap resource to track it
        let map_parent_entity = commands
            .spawn()
            .id();
        commands.insert_resource(CurrentMap {
            file: map_file.to_string(),
            map_handle,
            map_entity: map_parent_entity,
            terrain: Terrain::new(Vec3::new(16.0, 16.0, 16.0), UVec3::new(16, 16, 16))
        });

        state.push(AppState::MapLoadingFile).unwrap()
    }
}

// 1) When in LoadingMapState, checks if map finished loading
// 2) If so, loads tileset images, sets counter to 1 and goes to FiringEvents state
fn map_finish_loading(
    map_config: Res<MapConfig>,
    current_map: Res<CurrentMap>,
    asset_server: Res<AssetServer>,
    asset_server_settings: Res<AssetServerSettings>,
    mut state: ResMut<State<AppState>>,
    vidya_maps: Res<Assets<VidyaMap>>,
    mut commands: Commands
) {
    log::debug!("(SYSTEM) map_finish_loading");
    let load_state = asset_server.get_load_state(&current_map.map_handle);
    match load_state {
        LoadState::Loaded => {

            // Gets staged map and created its graphics
            let tiled_map = &vidya_maps
                .get(&current_map.map_handle)
                .unwrap()
                .tiled_map;
            let mut current_map_graphics = CurrentMapGraphics {
                chunk_size: map_config.chunk_size,
                ..Default::default()
            };

            // Begins loading map graphics asynchronously
            let asset_folder = PathBuf::from(&asset_server_settings.asset_folder);
            for tileset in tiled_map.tilesets() {
                if let Some(image) = &tileset.image {
                    let image_source = image.source.relativize(&asset_folder);
                    let image_handle = asset_server.load(image_source.as_path());
                    current_map_graphics
                        .tileset_image_handles
                        .push(Some(image_handle));
                }
                else {
                    current_map_graphics
                        .tileset_image_handles
                        .push(None);
                }
            }

            // Goes to next state
            commands.insert_resource(current_map_graphics);
            state.set(AppState::MapConstructing).unwrap();
        }
        LoadState::Failed => {
            panic!("Failed to load map file");
        }
        _ => {}
    }
}

// Constructs map
fn map_construct(
    mut current_map: ResMut<CurrentMap>,
    mut current_map_graphics: ResMut<CurrentMapGraphics>,
    vidya_map: Res<Assets<VidyaMap>>,
    mut state: ResMut<State<AppState>>,
    config: Res<MapConfig>
) {
    log::debug!("(SYSTEM) map_construct");
    
    // Gets tiled map
    let tiled_map = &vidya_map
        .get(&current_map.map_handle)
        .unwrap()
        .tiled_map;

    // Traverses the map and fires events based on its contents
    process_map(&tiled_map, config.flip_y, &mut current_map, &mut current_map_graphics).unwrap();
    state.set(AppState::MapSpawningEntities).unwrap();
}

fn map_spawn_entities(
    current_map: Res<CurrentMap>,
    current_map_graphics: ResMut<CurrentMapGraphics>,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<State<AppState>>,
    mut commands: Commands
) {
    log::debug!("(SYSTEM) map_spawn_entities");

    // Quits if graphics haven't finished loading yet
    let current_map_graphics = current_map_graphics.into_inner();
    if current_map_graphics.get_load_state(&assets) != LoadState::Loaded {
        return
    }

    // Spawns chunks as PBRBundles
    let map_entity = current_map.map_entity;
    let image_handles = &current_map_graphics.tileset_image_handles;
    for (key, chunk) in &current_map_graphics.chunks {

        // Try to get texture for current chunk
        let image_handle = match &image_handles[key.tileset_index] {
            Some(handle) => handle,
            None => return
        };

        // Creates mesh for chunk
        let chunk_size = current_map_graphics.chunk_size;
        let chunk_pos = Vec3::new(key.x as f32, key.y as f32, key.z as f32) * chunk_size;
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, chunk.positions.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, chunk.normals.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, chunk.uvs.clone());
        mesh.set_indices(Some(Indices::U32(chunk.indices.clone())));

        // Creates material for chunk
        let material = StandardMaterial {
            base_color_texture: Some(image_handle.clone()),
            metallic: 0.0,
            reflectance: 0.0,
            perceptual_roughness: 1.0,
            alpha_mode: AlphaMode::Mask(0.5),
            ..Default::default()
        };

        // Turns mesh and material into handles
        let mesh_handle = meshes.add(mesh);
        let material_handle = materials.add(material);

        // Spawns chunk as PbrBundle and attaches it to the map entity
        let chunk_entity = commands
            .spawn_bundle(PbrBundle {
                mesh: mesh_handle,
                material: material_handle,
                transform: Transform::from_translation(chunk_pos),
                ..Default::default()
            })
            .id();
        commands
            .entity(map_entity)
            .push_children(&[chunk_entity]);
    }

    // Spawns/configures lights
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: 27500.0,
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)).looking_towards(Vec3::new(0.0, -1.0, -1.0), Vec3::Y),
        ..Default::default()
    });

    // Spawns camera
    let cam_width = 800.0;
    let cam_height = 450.0;
    let cam_pos = Vec3::new(16.0*10.0, 1000.0, 600.0);
    let mut ortho_bundle = OrthographicCameraBundle::new_3d();
    let proj = &mut ortho_bundle.orthographic_projection;
    proj.scaling_mode = ScalingMode::WindowSize;
    proj.left = -cam_width / 2.0;
    proj.right = cam_width / 2.0;
    proj.bottom = -cam_height / 2.0;
    proj.top = cam_height / 2.0;
    proj.near = 1.0;
    proj.far = 10000.0;
    proj.scale = 0.5;
    ortho_bundle.transform = Transform::from_translation(cam_pos)
        .looking_towards(Vec3::new(0.0, -1.0, -1.0), Vec3::new(0.0, 1.0, 0.0))
        .with_scale(Vec3::new(1.0, 1.0/SQRT_2, 1.0));
    commands
        .spawn_bundle(CameraBundle::new(
            ortho_bundle,
            Position(cam_pos),
            Velocity(Vec3::ZERO),
            Friction { xz: 0.8, y: 0.8 },
            CameraTargetSettings { distance: 512.0 }
        ))
        .insert(Floater { speed: 2.0 });

    // Removes staging resources
    commands.remove_resource::<CurrentMap>();
    commands.remove_resource::<CurrentMapGraphics>();

    // Finishes map loading
    state.pop().unwrap();

    log::debug!("Done spawning map graphics entities...");
}
/// Map configuration resource
#[derive(Debug, PartialEq)]
pub struct MapConfig {
    pub chunk_size: Vec3,
    pub flip_y: bool
}