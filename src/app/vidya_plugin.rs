use std::iter::Iterator;
use std::path::PathBuf;
use bevy::asset::{AssetServerSettings, LoadState};
use bevy::prelude::*;
use tiled::*;
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
            .add_state(MapState::Absent)
            .add_event::<LoadMapEvent>()
            .add_event::<AddTileGraphicsEvent>()
            .add_asset::<VidyaMap>()
            .init_asset_loader::<VidyaMapLoader>()

            // App start
            .add_startup_system(start_app)

            // Map-loading (Fire LoadMapEvent to kickstart)
            .add_system_set(SystemSet::on_update(AppState::Running).with_system(on_load_map))                           // -> LoadingMap
            .add_system_set(SystemSet::on_update(MapState::LoadingMap).with_system(finish_loading_map))                 // -> PopulatingMap
            .add_system_set(SystemSet::on_enter(MapState::PopulatingMap).with_system(populate_map))                     // -> FinishLoadingMapGraphics
            .add_system_set(SystemSet::on_update(MapState::FinishLoadingMapGraphics).with_system(finish_loading_map_graphics))  // -> Finished
        ;
    }
    fn name(&self) -> &str { "vidya_plugin" }
}

// ---------------------- SYSTEMS ----------------------

fn start_app(mut state: ResMut<State<AppState>>) {
    // TODO: stuff
    state.set(AppState::Running).unwrap();
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
        for tileset in tiled_map.tilesets() {
            let image = tileset.image.as_ref().unwrap();
            let image_source = image.source.relativize(&asset_folder).display().to_string();
            let image_handle = asset_server.load(&image_source);
            current_map_graphics.tileset_image_handles.insert(tileset.name.clone(), image_handle);
        }
        state.set(MapState::PopulatingMap).unwrap();
        commands.insert_resource(current_map_graphics);
    }
}

// Fire events that cause map to populate
fn populate_map(
    current_map: Res<CurrentMap>,
    vidya_map: Res<Assets<VidyaMap>>,
    mut graphics_events: EventWriter<AddTileGraphicsEvent>,
    mut state: ResMut<State<MapState>>
) {
    // Gets tiled map
    let tiled_map = &vidya_map.get(&current_map.map_handle).unwrap().tiled_map;

    // For all group layers in the root...
    for root_layer in tiled_map.layers() {
        match &root_layer.layer_type() {
            LayerType::GroupLayer(group_layer) => {

                // Split the sub layers between terrain and collision
                let SplitGroupLayer {
                    terrain_layers,
                    collision_layer
                } = split_group_layer(group_layer);

                let offset = Vec2::new(
                    root_layer.data().offset_x,
                    root_layer.data().offset_y
                );

                // Populate tiles from group layer
                log::debug!("Processing group layer {}", &root_layer.data().name);
                add_tiles_from_group_layer(
                    tiled_map,
                    collision_layer,
                    terrain_layers.as_slice(),
                    offset,
                    &mut graphics_events
                );
            },
            _ => panic!("All root layers must be group layers")
        }
    }

    // Goes to state that waits for map graphics to finish loading
    state.set(MapState::FinishLoadingMapGraphics).unwrap();
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
        log::info!("Spawning map chunks for {}", current_map.file);
        state.set(MapState::Finished).unwrap();
    }
}


// ---------------------- UTIL STRUCTS/FUNCTIONS ----------------------

// A Group layer that has has been parsed (split between the terrain layers and the optional collision layer)
struct SplitGroupLayer<'map> {
    terrain_layers: Vec<TileLayer<'map>>,
    collision_layer: Option<TileLayer<'map>>
}

fn split_group_layer<'map>(group_layer: &'map GroupLayer<'map>) -> SplitGroupLayer<'map>{

    // Allocates
    let mut terrain_layers = Vec::new();
    let mut collision_layer = None;

    // Goes through sub layers and splits them
    for sub_layer in group_layer.layers() {
        let sub_properties = &sub_layer.data().properties;
        match sub_layer.layer_type() {
            LayerType::TileLayer(sub_layer) => {
                let tile_layer_type = get_string_property(sub_properties, "type").unwrap_or("terrain");
                match tile_layer_type {
                    "terrain" => terrain_layers.push(sub_layer),
                    "collision" => collision_layer = Some(sub_layer),
                    _ => panic!("Unexpected layer type '{}'", tile_layer_type)
                }
            },
            _ => { panic!("Sub layer must be a tile layer") }
        }
    }

    // Returns split data
    SplitGroupLayer {
        terrain_layers,
        collision_layer
    }
}


fn add_tiles_from_group_layer(
    map: &Map,                                              // Map itself
    c_layer: Option<TileLayer>,                             // Collision layer of group
    t_layers: &[TileLayer],                                 // Terrain layers of group
    offset: Vec2,                                           // Offset of collision layer
    graphics_events: &mut EventWriter<AddTileGraphicsEvent> // Resulting graphics events
) {
    // Prepares
    let (w, h) = (map.width as usize, map.height as usize);
    let (tw, th) = (map.tile_width as f32, map.tile_height as f32);
    let tile_size = Vec2::new(tw as f32, th as f32);

    // For all tiles in the group...
    for x in 0..w {
        let mut climb_status = ClimbStatus::NotClimbing;
        let mut current_pos = Vec3::new(
            x as f32 * tw + offset.x,
            offset.y,
            0.0
        );
        for y in (0..h).rev() {

            // Get collision/terrain tiles from group at x, y
            let c_tile = c_layer
                .as_ref()
                .and_then(|layer| layer.get_tile(x, y))
                .and_then(|l_tile| l_tile.get_tile());
            let t_tiles = t_layers
                .iter()
                .flat_map(|t_layer| t_layer.get_tile(x, y));

            // "Climb" the current tile at x, y
            log::debug!("x, y: {}, {}", x, y);
            log::debug!("Pos: {:?}", current_pos);
            let prev_status = climb_status;
            climb_status = add_tiles_with_collision_tile(
                map,
                c_tile,
                t_tiles,
                current_pos,
                tile_size,
                prev_status,
                graphics_events
            );
            log::debug!("Cur status: {:?}", climb_status);

            // Offset position for next iteration based on previous status and current one
            if climb_status == ClimbStatus::NotClimbing {
                current_pos.z -= th;
            }
            else if climb_status.is_climbing_wall() {
                current_pos.y += th;
            }
            else if climb_status == ClimbStatus::FinishedClimbing {
                let ydiff = current_pos.y - offset.y;
                current_pos.y = offset.y;
                current_pos.z -= ydiff + th;
            }
        }

    }
}

fn add_tiles_with_collision_tile<'map>(
    map: &'map Map,
    c_tile: Option<&'map Tile>,
    t_tiles: impl Iterator<Item=LayerTile<'map>>,
    position: Vec3,
    size: Vec2,
    prev_climb_status: ClimbStatus,
    graphics_events: &mut EventWriter<AddTileGraphicsEvent>,
) -> ClimbStatus {

    // Gets collision type type of collision tile
    //let collision_type = get_string_property(&c_tile.properties, "type").unwrap_or("floor");
    let collision_type = c_tile
        .map(|t| &t.properties)
        .and_then(|p| get_string_property(p, "type"))
        .unwrap_or("floor");
    let collision_type = CollisionType::from_str(collision_type).unwrap();
    let tilesets = map.tilesets();

    // Determines next climb status and terrain shape
    let next_status = ClimbStatus::next(prev_climb_status, collision_type);
    let shape = next_status.to_terrain_shape();

    // For all terrain layers belonging to the same layer group...
    for t_tile in t_tiles {
        let tileset_index = tilesets
            .iter()
            .position(|tileset| &tileset.name == &t_tile.tileset.name)
            .unwrap() as u32;

        // Get the terrain tile from this layer that is in the same position as the collision tile
        let event = AddTileGraphicsEvent(TileGraphics {
            tileset_index,
            tile_index: t_tile.id as u32,
            position,
            size,
            shape
        });

        // Send event for adding tile's graphics
        log::debug!("Fired event {:?}", event);
        graphics_events.send(event);
    }

    // Done
    next_status
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum ClimbStatus {
    NotClimbing,
    ClimbingWallS,
    ClimbingWallSE,
    ClimbingWallSW,
    FinishedClimbing
}

impl ClimbStatus {
    
    fn next(prev_status: ClimbStatus, collision: CollisionType) -> Self {
        log::debug!("Type: {:?}, prev stat: {:?}", collision, prev_status);
        // What should the resulting climb status be, considering the current collision tile and the previous climb status?
        // Yes, this is ugly and no, I'm not going to fix it...
        if collision == CollisionType::Floor {
            let is_status_valid =
                prev_status == Self::NotClimbing ||
                prev_status == Self::ClimbingWallS ||
                prev_status == Self::ClimbingWallSE ||
                prev_status == Self::ClimbingWallSW ||
                prev_status == Self::FinishedClimbing;
            if !is_status_valid {
                panic!("Encountered a {:?} tile while in climb status {:?}", collision, prev_status)
            }
            Self::NotClimbing
        }
        else if collision == CollisionType::Wall {
            if  prev_status == Self::NotClimbing ||
                prev_status == Self::ClimbingWallS ||
                prev_status == Self::FinishedClimbing {
                Self::ClimbingWallS
            }
            else if prev_status == Self::ClimbingWallSE {
                Self::ClimbingWallSE
            }
            else if prev_status == Self::ClimbingWallSW {
                Self::ClimbingWallSW
            }
            else {
                // Slopes???
                todo!()
            }
        }
        else if collision == CollisionType::WallStartSE {
            let is_status_valid =
                prev_status == Self::NotClimbing ||
                prev_status == Self::FinishedClimbing;
            if !is_status_valid {
                panic!("Encountered a {:?} tile while in climb status {:?}", collision, prev_status)
            }
            Self::ClimbingWallSE
        }
        else if collision == CollisionType::WallStartSW {
            let is_status_valid =
                prev_status == Self::NotClimbing ||
                prev_status == Self::FinishedClimbing;
            if !is_status_valid {
                panic!("Encountered a {:?} tile while in climb status {:?}", collision, prev_status)
            }
            Self::ClimbingWallSW
        }
        else if collision == CollisionType::WallEndSE {
            if prev_status != Self::ClimbingWallSE {
                panic!("Encountered a {:?} tile while in climb status {:?}", collision, prev_status)
            }
            Self::NotClimbing
        }
        else if collision == CollisionType::WallEndSW {
            if prev_status != Self::ClimbingWallSW {
                panic!("Encountered a {:?} tile while in climb status {:?}", collision, prev_status)
            }
            Self::NotClimbing
        }
        else if collision.is_lip() {
            if !(prev_status.is_climbing_wall() || prev_status == Self::NotClimbing) {
                panic!("Encountered a {:?} tile while in climb status {:?}", collision, prev_status)
            }
            Self::FinishedClimbing
        }
        else {
            // Slopes???
            todo!()
        }
    }

    fn is_climbing_wall(self) -> bool {
        self == Self::ClimbingWallS ||
        self == Self::ClimbingWallSE ||
        self == Self::ClimbingWallSW
    }

    fn to_terrain_shape(self) -> TerrainShape {
        match self {
            Self::NotClimbing => TerrainShape::Floor,
            Self::FinishedClimbing => TerrainShape::Floor,
            Self::ClimbingWallS => TerrainShape::Wall,
            Self::ClimbingWallSE => TerrainShape::WallSE,
            Self::ClimbingWallSW => TerrainShape::WallSW
        }
    }
}

fn get_string_property<'a>(properties: &'a Properties, key: &str) -> Option<&'a str> {
    match properties.get(key) {
        Some(PropertyValue::StringValue(value)) => Some(value),
        _ => None
    }
}