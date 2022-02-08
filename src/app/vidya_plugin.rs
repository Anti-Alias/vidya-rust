use std::path::PathBuf;
use bevy::asset::{AssetServerSettings, LoadState};
use bevy::prelude::*;
use bevy::utils::HashMap;
use crate::map::{CurrentMap, CurrentMapGraphics, LoadMapEvent, MapState, VidyaMap, VidyaMapLoader};
use crate::extensions::*;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    Starting,
    Started
}

#[derive(Default)]
pub struct VidyaPlugin;
impl Plugin for VidyaPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_state(MapState::Absent)
            .add_state(AppState::Starting)
            .add_event::<LoadMapEvent>()
            .add_asset::<VidyaMap>()
            .init_asset_loader::<VidyaMapLoader>()
            .add_system(on_load_map)
            .add_system_set(SystemSet::on_update(MapState::LoadingMap).with_system(finish_loading_map))
            .add_system_set(SystemSet::on_update(MapState::LoadingMapGraphics).with_system(finish_loading_map_graphics))
            .add_system_set(SystemSet::on_enter(MapState::PopulatingMap).with_system(populate_map))
        ;
    }
    fn name(&self) -> &str { "vidya_plugin" }
}

// 1) Listens for "LoadMapEvent"
// 2) Begins loading map specified
// 3) Goes to LoadingMap state
fn on_load_map(
    mut events: EventReader<LoadMapEvent>,
    mut map_state: ResMut<State<MapState>>,
    current_map: Option<ResMut<CurrentMap>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands
) {
    if let Some(event) = events.iter().next() {

        // Begins loading map and stores the handle for later use
        let map_file = &event.0;
        let map_handle = asset_server.load(map_file);

        // If we already have a current map, despawn the current map entity
        if let Some(current_map) = current_map {
            commands
                .entity(current_map.map_entity)
                .despawn();
            commands.remove_resource::<CurrentMap>();
            log::info!("Despawned map {}", current_map.file);
        }

        // Spawns map parent entity
        let map_parent_entity = commands
            .spawn()
            .id();

        // Tracks current map in a resource and its graphics
        commands.insert_resource(CurrentMap {
            file: map_file.to_string(),
            map_handle,
            map_entity: map_parent_entity
        });

        // Puts game in loading state
        map_state.set(MapState::LoadingMap);
        log::info!("Loading map {}", map_file);
    }
}

// 1) When in LoadingMapState, checks if map finished loading
// 2) If so, loads tileset images and goes to LoadingMapGraphics state
fn finish_loading_map(
    mut current_map: ResMut<CurrentMap>,
    mut asset_server: ResMut<AssetServer>,
    asset_server_settings: Res<AssetServerSettings>,
    mut state: ResMut<State<MapState>>,
    vidya_maps: Res<Assets<VidyaMap>>,
    mut commands: Commands
) {
    if asset_server.get_load_state(&current_map.map_handle) == LoadState::Loaded {

        // Creates initial map graphics
        log::info!("Map {} finished loading", &current_map.file);
        let mut current_map_graphics = CurrentMapGraphics {
            chunk_width: 32,
            chunk_height: 32,
            tile_width: 16,
            tile_height: 16,
            ..Default::default()
        };

        // Gets parent directory of tmx map file
        let tiled_map = &vidya_maps
            .get(&current_map.map_handle)
            .unwrap()
            .tiled_map;

        // Begins loading map graphics
        let asset_folder = PathBuf::from(&asset_server_settings.asset_folder);
        for tileset in &tiled_map.tilesets {
            let image = tileset.image.as_ref().unwrap();
            let image_source = image.source.relativize(&asset_folder).display().to_string();
            let image_handle: Handle<Image> = asset_server.load(&image_source);
            current_map_graphics.tileset_image_handles.insert(tileset.first_gid, image_handle);
        }
        state.set(MapState::LoadingMapGraphics);
        commands.insert_resource(current_map_graphics);
    }
}

// 1) When in LoadingMapGraphics state, checks if tilesets finished loading
// 2) If so, goes to SpawningMapChunks state
fn finish_loading_map_graphics(
    current_map: Res<CurrentMap>,
    current_map_graphics: Res<CurrentMapGraphics>,
    asset_server: Res<AssetServer>,
    mut state: ResMut<State<MapState>>
) {
    // If tileset images have finished loading, go into spawning state
    let tileset_handles = current_map_graphics.tileset_image_handles.values().map(|handle| { handle.id });
    if asset_server.get_group_load_state(tileset_handles) == LoadState::Loaded {
        log::info!("Finished loading tilesets for {}", current_map.file);
        log::info!("Spawning map chunks for {}", current_map.file);
        state.set(MapState::PopulatingMap);
    }
}

// Fire events that cause map to populate
fn populate_map(
    current_map: Res<CurrentMap>,
    current_map_graphics: Res<CurrentMapGraphics>,
    asset_server: Res<AssetServer>,
    vidya_map: Res<Assets<VidyaMap>>,
    mut state: ResMut<State<MapState>>
) {
    let tiled_map = &vidya_map.get(&current_map.map_handle).unwrap().tiled_map;
    for (id, layer) in tiled_map.layers.iter().enumerate() {
    }
    state.set(MapState::Finished);
}

