use std::path::PathBuf;
use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy::utils::HashMap;
use crate::map::{CurrentMap, LoadMapEvent, MapState, VidyaMap, VidyaMapLoader};

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
            .add_system(on_new_map)
            .add_system_set(SystemSet::on_update(MapState::LoadingMap).with_system(check_loading_map))
            .add_system_set(SystemSet::on_enter(MapState::LoadingMapGraphics).with_system(load_map_graphics))
        ;
    }
    fn name(&self) -> &str { "vidya_plugin" }
}

fn on_new_map(
    mut events: EventReader<LoadMapEvent>,
    mut map_state: ResMut<State<MapState>>,
    current_map: Option<ResMut<CurrentMap>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands
) {
    if let Some(event) = events.iter().next() {

        // Begins loading map and stores the handle for later use
        let map_name = &event.0;
        let map_handle = asset_server.load(map_name);

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

        // Tracks current map in a temp resource
        commands.insert_resource(CurrentMap {
            file: map_name.to_string(),
            map_handle: map_handle,
            tileset_handles: HashMap::default(),
            map_entity: map_parent_entity
        });

        // Puts us in loading state
        map_state.set(MapState::LoadingMap);
        log::info!("Loading map {}", map_name);
    }
}

fn check_loading_map(
    mut asset_server: ResMut<AssetServer>,
    current_map: Res<CurrentMap>,
    mut state: ResMut<State<MapState>>
) {
    if asset_server.get_load_state(&current_map.map_handle) == LoadState::Loaded {
        state.set(MapState::LoadingMapGraphics);
        log::info!("Map {} finished loading", &current_map.file);
    }
}

fn load_map_graphics(
    mut current_map: ResMut<CurrentMap>,
    mut state: ResMut<State<MapState>>,
    vidya_maps: Res<Assets<VidyaMap>>,
    mut asset_server: ResMut<AssetServer>
) {
    let tiled_map = &vidya_maps
        .get(&current_map.map_handle)
        .unwrap()
        .tiled_map;
    let map_file = PathBuf::from(&current_map.file);
    let map_dir = map_file.parent().unwrap();
    for tileset in &tiled_map.tilesets {
        let map_image = &tileset.images[0];
        let image_path = format!("{}/{}", map_dir.display(), map_image.source);
        let image_handle: Handle<Image> = asset_server.load(&image_path);
        current_map.tileset_handles.insert(tileset.first_gid, image_handle);
        log::info!("Loading tileset {}", image_path);
    }
}

fn check_loading_map_graphics (

) {

}