use std::f32::consts::SQRT_2;
use std::iter::Iterator;
use std::path::PathBuf;
use std::sync::{ Mutex };

use crate::app::climb::add_tiles_from_map;
use crate::map::*;
use crate::extensions::*;

use bevy::asset::{ AssetServerSettings, LoadState };
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;


#[derive(Default)]
pub struct VidyaPlugin;
impl Plugin for VidyaPlugin {

    fn build(&self, app: &mut App) {
        app
            .add_state(AppState::AppStarting)
            .add_event::<LoadMapEvent>()
            .add_event::<AddTileGraphicsEvent>()
            .add_asset::<VidyaMap>()
            .init_asset_loader::<VidyaMapLoader>()
            .insert_resource(MapConfig {
                chunk_size: Vec3::new(
                    (16*16) as f32,
                    (16*16) as f32,
                    (16*16) as f32
                )
            })

            // App systems
            .add_startup_system(start_app)


            // Map systems
            .add_system_set(SystemSet::on_update(AppState::AppRunning)
                .with_system(map_listen)
            )
            .add_system_set(SystemSet::on_update(AppState::MapLoadingFile)
                .with_system(map_finish_loading_file_client)                    // Sets counter to 2
            )
            .add_system_set(SystemSet::on_enter(AppState::MapFiringEvents)
                .with_system(map_fire_events)
            )
            .add_system_set(SystemSet::on_enter(AppState::MapHandlingEvents)
                .with_system(map_handle_events)
                .with_system(map_handle_graphics_events)
            )
            .add_system_set(SystemSet::on_enter(AppState::MapSpawningEntities)
                .with_system(map_spawn_graphics_entities)
                .with_system(map_spawn_collision_entities)                      // Decrements counter
                .with_system(map_goto_finishing)
            )
            .add_system_set(SystemSet::on_update(AppState::MapFinishing)
                .with_system(map_finish_loading_assets)                         // Decrements counter
            )
            
        ;
    }
    fn name(&self) -> &str { "vidya_plugin" }
}

// ---------------------- SYSTEMS ----------------------

fn start_app(mut app_state: ResMut<State<AppState>>) {
    // TODO: stuff
    log::debug!("Entered system 'start_app'");
    app_state.set(AppState::AppRunning).unwrap();
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
    log::debug!("Entered system 'on_load_map_event'");
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
            counter: Mutex::new(0)
        });

        state.push(AppState::MapLoadingFile).unwrap()
    }
}

// 1) When in LoadingMapState, checks if map finished loading
// 2) If so, loads tileset images, sets counter to 1 and goes to FiringEvents state
fn map_finish_loading_file_client(
    map_config: Res<MapConfig>,
    current_map: Res<CurrentMap>,
    asset_server: Res<AssetServer>,
    asset_server_settings: Res<AssetServerSettings>,
    mut state: ResMut<State<AppState>>,
    vidya_maps: Res<Assets<VidyaMap>>,
    mut commands: Commands
) {
    log::debug!("Entered system 'finish_loading_map_file_client'");
    let load_state = asset_server.get_load_state(&current_map.map_handle);
    match load_state {
        LoadState::Loaded => {

            // Sets the counter to 2, so we wait for both the collision and graphics to spawn
            {
                let mut counter = current_map.counter.lock().unwrap();
                *counter = 2;
                log::debug!("Counter is {}", counter);
            }

            // Gets parent directory of tmx map file
            let tiled_map = &vidya_maps
                .get(&current_map.map_handle)
                .unwrap()
                .tiled_map;

            // Creates initial map graphics resource
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
            state.set(AppState::MapFiringEvents).unwrap();
            log::debug!("Added map graphics");
        }
        LoadState::Failed => {
            panic!("Failed to load map file");
        }
        _ => {}
    }
}

// 1) When in LoadingMapState, checks if map finished loading
// 2) If so, loads tileset images, sets counter to 2 and goes to FiringEvents state
#[allow(dead_code)]
fn finish_loading_map_file_server(
    current_map: Res<CurrentMap>,
    asset_server: Res<AssetServer>,
    mut state: ResMut<State<AppState>>
) {
    log::debug!("Entered system 'finish_loading_map_file_server'");
    if asset_server.get_load_state(&current_map.map_handle) == LoadState::Loaded {
        log::debug!("Map {} finished loading", &current_map.file);
        {
            let mut counter = current_map.counter.lock().unwrap();
            *counter = 1;
            log::debug!("Counter is {}", counter);
        }
        state.set(AppState::MapFiringEvents).unwrap();
    }
}

// Fire events that cause map to populate
fn map_fire_events(
    current_map: Res<CurrentMap>,
    vidya_map: Res<Assets<VidyaMap>>,
    graphics_events: EventWriter<AddTileGraphicsEvent>,
    mut state: ResMut<State<AppState>>
) {
    log::debug!("Entered system 'fire_map_events'");
    // Gets tiled map
    let tiled_map = &vidya_map
        .get(&current_map.map_handle)
        .unwrap()
        .tiled_map;

    // "Climbs" all group layers of map and fires events
    add_tiles_from_map(&tiled_map, graphics_events, true);

    // Goes to state that waits for map graphics to finish loading
    state.set(AppState::MapHandlingEvents).unwrap();
}

fn map_handle_graphics_events(
    mut graphics_events: EventReader<AddTileGraphicsEvent>,
    mut current_map_graphics: ResMut<CurrentMapGraphics>
) {
    log::debug!("Entered system 'handle_map_graphics'");
    for event in graphics_events.iter() {
        current_map_graphics.add_tile(event.0);
    }
}

fn map_handle_events(mut map_state: ResMut<State<AppState>>) {
    log::debug!("Entered system 'on_load_map_event'");
    map_state.set(AppState::MapSpawningEntities).unwrap();
}

fn map_finish_loading_assets(
    current_map: Res<CurrentMap>,
    current_map_graphics: Option<Res<CurrentMapGraphics>>,
    asset_server: Res<AssetServer>,
    mut state: ResMut<State<AppState>>,
    mut commands: Commands
) {
    log::debug!("Entered system 'map_finish_loading_graphics'");
    if let Some(current_map_graphics) = current_map_graphics {
        // If tileset images have finished loading, go into spawning state
        let handle_ids = current_map_graphics
            .tileset_image_handles
            .iter()
            .flatten()
            .map(|handle| { handle.id });
        if asset_server.get_group_load_state(handle_ids) == LoadState::Loaded {
            commands.remove_resource::<CurrentMapGraphics>();
            let mut counter = current_map.counter.lock().unwrap();
            *counter -= 1;
            state.pop().unwrap();
            log::debug!("Counter is {}", counter);
        }
    }
}

#[allow(dead_code)]
fn map_spawn_graphics_entities(
    current_map: Res<CurrentMap>,
    current_map_graphics: Res<CurrentMapGraphics>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands
) {
    log::debug!("Entered system 'spawn_map_graphics_entities'");
    let map_entity = current_map.map_entity;
    let image_handles = &current_map_graphics.tileset_image_handles;

    // Spawns chunks as PBRBundles
    for (key, chunk) in &current_map_graphics.chunks {

        // Try to get texture for current chunk
        if let Some(image_handle) = &image_handles[key.tileset_index] {

            // Creates mesh
            let chunk_size = current_map_graphics.chunk_size;
            let chunk_pos = Vec3::new(key.x as f32, key.y as f32, key.z as f32) * chunk_size;
            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
            mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, chunk.positions.clone());
            mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, chunk.normals.clone());
            mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, chunk.uvs.clone());
            mesh.set_indices(Some(Indices::U32(chunk.indices.clone())));

            // Creates material
            let material = StandardMaterial {
                base_color_texture: Some(image_handle.clone()),
                metallic: 0.0,
                reflectance: 0.0,
                unlit: true,
                ..Default::default()
            };

            // Turns mesh and material into handles
            let mesh_handle = meshes.add(mesh);
            let material_handle = materials.add(material);

            // Spawns chunk as PbrBundle and attaches it to the map entity
            let chunk_entity = commands.spawn_bundle(PbrBundle {
                mesh: mesh_handle,
                material: material_handle,
                transform: Transform::from_translation(chunk_pos),
                ..Default::default()
            }).id();
            commands.entity(map_entity).push_children(&[chunk_entity]);
        }
    }

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
    log::debug!("Done spawning map graphics entities...");
}

// TODO
fn map_spawn_collision_entities(current_map: Res<CurrentMap>) {
    let mut counter = current_map.counter.lock().unwrap();
    *counter -= 1;
    log::debug!("In system 'spawn_map_collision_entities'");
    log::debug!("Counter is {}", counter);
}

fn map_goto_finishing(mut state: ResMut<State<AppState>>) {
    state.set(AppState::MapFinishing).unwrap();
}


/// High-level state of the application as a whole
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {

    // App events
    AppStarting,
    AppRunning,
    AppStopped,

    // Map events
    MapLoadingFile,
    MapFiringEvents,
    MapHandlingEvents,
    MapSpawningEntities,
    MapFinishing
}

/// Map configuration resource
#[derive(Debug, PartialEq)]
pub struct MapConfig {
    pub chunk_size: Vec3
}