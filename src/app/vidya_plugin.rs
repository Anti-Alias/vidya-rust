use std::iter::Iterator;
use std::path::PathBuf;
use bevy::asset::{AssetServerSettings, LoadState};
use bevy::prelude::*;
use crate::app::climb::add_tiles_from_map;
use crate::map::*;
use crate::extensions::*;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    Starting,
    Running,
    Stopped
}

pub struct NoMoving;

#[derive(Default)]
pub struct VidyaPlugin;
impl Plugin for VidyaPlugin {

    fn build(&self, app: &mut App) {
        app
            .add_state(AppState::Starting)
            .add_state(MapState::Listening)
            .add_event::<LoadMapEvent>()
            .add_event::<AddTileGraphicsEvent>()
            .add_asset::<VidyaMap>()
            .init_asset_loader::<VidyaMapLoader>()

            // App start
            .add_startup_system(start_app)

            // Map-loading (Fire LoadMapEvent to kickstart)
            .add_system_set(SystemSet::on_update(AppState::Running)
                .with_system(on_load_map)
            )
            .add_system_set(SystemSet::on_update(MapState::LoadingMap)
                .with_system(finish_loading_map)
            )
            .add_system_set(SystemSet::on_enter(MapState::FiringMapEvents)
                .with_system(fire_map_events)
            )
            .add_system_set(SystemSet::on_enter(MapState::HandlingMapEvents)
                .with_system(handle_graphics_events)
                .with_system(handle_collision_events)
            )
            .add_system_set(SystemSet::on_update(MapState::FinishingLoadingMapGraphics)
                .with_system(finish_loading_map_graphics)
            );
    }
    fn name(&self) -> &str { "vidya_plugin" }
}

// ---------------------- SYSTEMS ----------------------

fn start_app(mut app_state: ResMut<State<AppState>>) {
    // TODO: stuff
    app_state.set(AppState::Running).unwrap();
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
        map_state.set(MapState::LoadingMap).unwrap();
        log::info!("Loading map {}", map_file);
    }
}

// 1) When in LoadingMapState, checks if map finished loading
// 2) If so, loads tileset images and goes to LoadingMapGraphics state
fn finish_loading_map(
    current_map: Res<CurrentMap>,
    asset_server: Res<AssetServer>,
    asset_server_settings: Res<AssetServerSettings>,
    mut state: ResMut<State<MapState>>,
    vidya_maps: Res<Assets<VidyaMap>>,
    mut commands: Commands
) {
    if asset_server.get_load_state(&current_map.map_handle) == LoadState::Loaded {

        // Creates initial map graphics
        log::info!("Map {} finished loading", &current_map.file);
        let mut current_map_graphics = CurrentMapGraphics {
            chunk_size: Vec3::new(256.0, 256.0, 256.0),
            ..Default::default()
        };

        // Gets parent directory of tmx map file
        let tiled_map = &vidya_maps
            .get(&current_map.map_handle)
            .unwrap()
            .tiled_map;

        // Begins loading map graphics
        let asset_folder = PathBuf::from(&asset_server_settings.asset_folder);
        for tileset in tiled_map.tilesets() {
            let image = tileset.image.as_ref().unwrap();
            let image_source = image.source.relativize(&asset_folder);
            let image_handle = asset_server.load(image_source.as_path());
            current_map_graphics.tileset_image_handles.insert(tileset.name.clone(), image_handle);
        }
        state.set(MapState::FiringMapEvents).unwrap();
        commands.insert_resource(current_map_graphics);
    }
}

// Fire events that cause map to populate
fn fire_map_events(
    current_map: Res<CurrentMap>,
    vidya_map: Res<Assets<VidyaMap>>,
    graphics_events: EventWriter<AddTileGraphicsEvent>,
    mut state: ResMut<State<MapState>>
) {
    // Gets tiled map
    let tiled_map = &vidya_map
        .get(&current_map.map_handle)
        .unwrap()
        .tiled_map;

    // "Climbs" all group layers of map and fires events
    add_tiles_from_map(&tiled_map, graphics_events, true);

    // Goes to state that waits for map graphics to finish loading
    state.set(MapState::HandlingMapEvents).unwrap();
}

fn handle_graphics_events(
    mut graphics_events: EventReader<AddTileGraphicsEvent>,
    mut current_map_graphics: ResMut<CurrentMapGraphics>
) {
    log::info!("Handling graphics events...");
    for event in graphics_events.iter() {
        current_map_graphics.add_tile(event.0);
    }
}

fn handle_collision_events() {

}

fn finish_loading_map_graphics(
    current_map: Res<CurrentMap>,
    current_map_graphics: Res<CurrentMapGraphics>,
    asset_server: Res<AssetServer>,
    mut state: ResMut<State<MapState>>
) {
    // If tileset images have finished loading, go into spawning state
    let handle_ids = current_map_graphics
        .tileset_image_handles
        .values()
        .map(|handle| { handle.id });
    if asset_server.get_group_load_state(handle_ids) == LoadState::Loaded {
        log::info!("Finished loading tilesets for {}", current_map.file);
        state.set(MapState::Listening).unwrap();
    }
}