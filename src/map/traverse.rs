use bevy::prelude::*;
use tiled::*;
use std::result::Result;

use crate::map::{ GeomShape, TileType, TileGraphics, TileMeshData };
use super::{CurrentMapGraphics, CurrentMap};

const DEPTH_EPSILON: f32 = 0.001;


macro_rules! climb_panic {
    ($tile_type:expr, $prev_status:expr, $group_layer_name:expr, $x:expr, $y:expr) => {
        panic!(
            "Encountered a {:?} tile while in climb status {:?} on group layer '{}' at {}, {}",
            $tile_type,
            $prev_status,
            $group_layer_name,
            $x,
            $y
        );
    }
}



// Fire events that cause map to populate
pub(crate) fn traverse_map(
    tiled_map: &tiled::Map,
    flip_y: bool,
    current_map: &mut CurrentMap,
    current_map_graphics: &mut CurrentMapGraphics
) -> Result<(), ClimbingError> {
    // For all group layers in the root...
    let mut flattened_layer_index = 0;
    for root_layer in tiled_map.layers() {
        match &root_layer.layer_type() {
            LayerType::GroupLayer(group_layer) => {

                // Split the sub layers between terrain and meta layers
                let SplitGroupLayer {
                    terrain_layers,
                    meta_layers
                } = split_group_layer(group_layer);


                // Populate tiles from group layer
                log::trace!("Processing group layer {}", &root_layer.name);
                let offset = Vec2::new(
                    root_layer.offset_x,
                    -root_layer.offset_y
                );
                traverse_group_layer(
                    &meta_layers,
                    &terrain_layers,
                    offset,
                    tiled_map,
                    flip_y,
                    &root_layer.name,
                    current_map,
                    current_map_graphics,
                    flattened_layer_index
                )?;
                flattened_layer_index += terrain_layers.len();
            },
            _ => return Err(ClimbingError("All root layers must be group layers".to_owned()))
        }
    }
    Ok(())
}


fn traverse_group_layer(
    m_layers: &[MetaLayer],                                     // Group meta layers
    t_layers: &[TileLayer],                                     // Group terrain layers
    offset: Vec2,                                               // Group offset
    map: &Map,                                                  // Map itself
    flip_y: bool,
    group_layer_name: &str,
    current_map: &mut CurrentMap,
    current_map_graphics: &mut CurrentMapGraphics,
    flattened_layer_index: usize,
) -> Result<(), ClimbingError> {
    // For all columns in the group...
    let (w, h) = (map.width, map.height);
    let tile_size = Vec2::new(map.tile_width as f32, map.tile_height as f32);
    for x in 0..w {

        // Make climbers at the bottom of the vertical strip...
        let c_pos = Vec2::new(x as f32, 0.0) * tile_size + offset;
        let mut geom_climber = Climber::new(c_pos, tile_size.y);
        let mut coll_climber = Climber::new(c_pos, tile_size.y);

        // Traverse the strip from bottom to top
        let x = x as i32;
        for y in (0..h).rev() {

            // Gets meta tile (optional) and terrain tiles at (x, y)
            let y = y as i32;

            // "Climb" the current tile at x, y
            add_tiles(
                m_layers,
                t_layers,
                x,
                y,
                &mut geom_climber,
                &mut coll_climber,
                flip_y,
                group_layer_name,
                current_map,
                current_map_graphics,
                flattened_layer_index
            )?;
        }
    }
    Ok(())
}

fn add_tiles<'map>(
    meta_layers: &[MetaLayer<'map>],
    terrain_layers: &[TileLayer],
    tile_x: i32,
    tile_y: i32,
    geom_climber: &mut Climber,
    coll_climber: &mut Climber,
    flip_y: bool,
    group_layer_name: &str,
    _current_map: &mut CurrentMap,
    current_map_graphics: &mut CurrentMapGraphics,
    flattened_layer_index: usize
) -> Result<(), ClimbingError> {

    // Gets first meta and terrain tiles found at tile_x, tile_y
    let meta_tile = meta_layers
        .iter()
        .flat_map(|m_layer| m_layer.get_tile(tile_x, tile_y))
        .next();
    let terrain_tiles = terrain_layers
        .iter()
        .flat_map(|layer| layer.get_tile(tile_x, tile_y));

    // Gets the geom/coll types of current meta tile and uses it to "climb" both the collision and geometry
    let (geom_type, coll_type) = meta_tile
        .map(|tile| tile.get_types())
        .unwrap_or((TileType::Floor, TileType::Floor));
    let geom_pos = geom_climber.position;

    let prev_geom_status = geom_climber.climb_status;
    geom_climber.climb(geom_type, tile_x, tile_y, group_layer_name);
    coll_climber.climb(coll_type, tile_x, tile_y, group_layer_name);
    let geom_shape = geom_climber.geom_shape(prev_geom_status)?;

    // For all terrain layers belonging to the same layer group in the same position...
    for (layer_index, t_tile) in terrain_tiles.enumerate() {

        // Finds tileset, and computes mesh data
        let tileset_index = t_tile.tileset_index();
        let tileset = t_tile.get_tileset();
        let tile_mesh_data = get_tile_mesh_data(&tileset, t_tile.id(), flip_y);
        let flattened_layer_index = flattened_layer_index + layer_index;
        let depth_offset = Vec3::new(0.0, DEPTH_EPSILON, DEPTH_EPSILON) * flattened_layer_index as f32;

        // Add tile info to results
        current_map_graphics.add_tile(TileGraphics {
            tileset_index: tileset_index as u32,
            translation: geom_pos + depth_offset,
            mesh_data: tile_mesh_data,
            shape: geom_shape
        });
    }
    Ok(())
}

fn split_group_layer<'map>(group_layer: &'map GroupLayer<'map>) -> SplitGroupLayer<'map>{

    // Goes through sub layers and splits them
    let mut terrain_layers = Vec::new();
    let mut meta_layers = Vec::new();
    for sub_layer in group_layer.layers() {
        let sub_properties = &sub_layer.properties;
        match sub_layer.layer_type() {
            LayerType::TileLayer(sub_layer) => {
                let tile_layer_type = get_string_property(sub_properties, "type").unwrap_or("terrain");
                match tile_layer_type {
                    "terrain" => terrain_layers.push(sub_layer),
                    "geom_coll" => meta_layers.push(MetaLayer::GeomColl(sub_layer)),
                    "geom" => meta_layers.push(MetaLayer::Geom(sub_layer)),
                    "coll" => meta_layers.push(MetaLayer::Coll(sub_layer)),
                    _ => panic!("Unexpected tile layer type '{}'", tile_layer_type)
                }
            },
            _ => panic!("Sub layer must be a tile layer")
        }
    }

    // Returns split data
    SplitGroupLayer { terrain_layers, meta_layers }
}


/// Information about a tile that was just climbed in the map
#[derive(Debug, Copy, Clone)]
pub struct TileInfo {
    pub graphics: TileGraphics
}

/// Determines the status of a climb.
/// Used in conjunction with `Climber`
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum ClimbStatus {
    /// Tile is treated as a floor. Next tile is 1 farther (z)
    NotClimbing,
    // Tile is treated as a wall. Next tile is 1 higher (y)
    ClimbingWallS,
    /// Tile is treated as a south-east wall. Next tile is 1 higher (y)
    ClimbingWallSE,
    // Tile is treated as a south-west wall. Next tile is 1 higher (y)
    ClimbingWallSW,
    // Just encountered a "lip" tile. Tile is treated as floor. Next tile is N tiles below (y) and N tiles farther (z), where N represents how high we were in tiles
    FinishedClimbing
}

impl ClimbStatus {
    
    /// Takes into account the climb status of the previous tile and the type of the current tile to determine the
    /// climb status for the current tile.
    fn next(
        prev_status: ClimbStatus,
        tile_type: TileType,
        tile_x: i32,
        tile_y: i32,
        group_layer_name: &str
    ) -> Self {
        // What should the resulting climb status be, considering the current collision tile and the previous climb status?
        // Yes, this is ugly and no, I'm not going to fix it...
        if tile_type == TileType::Floor {
            let is_status_valid =
                prev_status == Self::NotClimbing ||
                prev_status == Self::ClimbingWallS ||
                prev_status == Self::ClimbingWallSE ||
                prev_status == Self::ClimbingWallSW ||
                prev_status == Self::FinishedClimbing;
            if !is_status_valid {
                //climb_panic!(tile_type, prev_status, tile_x, tile_y, group_layer_name);
                climb_panic!(tile_type, prev_status, group_layer_name, tile_x, tile_y);
            }
            Self::NotClimbing
        }
        else if tile_type == TileType::Wall {
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
        else if tile_type == TileType::WallStartSE {
            let is_status_valid =
                prev_status == Self::NotClimbing ||
                prev_status == Self::FinishedClimbing;
            if !is_status_valid {
                climb_panic!(tile_type, prev_status, group_layer_name, tile_x, tile_y);
            }
            Self::ClimbingWallSE
        }
        else if tile_type == TileType::WallStartSW {
            let is_status_valid =
                prev_status == Self::NotClimbing ||
                prev_status == Self::FinishedClimbing;
            if !is_status_valid {
                climb_panic!(tile_type, prev_status, group_layer_name, tile_x, tile_y);
            }
            Self::ClimbingWallSW
        }
        else if tile_type == TileType::WallEndSE {
            if prev_status != Self::ClimbingWallSE {
                climb_panic!(tile_type, prev_status, group_layer_name, tile_x, tile_y);
            }
            Self::NotClimbing
        }
        else if tile_type == TileType::WallEndSW {
            if prev_status != Self::ClimbingWallSW {
                climb_panic!(tile_type, prev_status, group_layer_name, tile_x, tile_y);
            }
            Self::NotClimbing
        }
        else if tile_type.is_lip() {
            if !(prev_status.is_climbing_wall() || prev_status == Self::NotClimbing) {
                climb_panic!(tile_type, prev_status, group_layer_name, tile_x, tile_y);
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
}

// Helper function that assumes a property is a string
fn get_string_property<'a>(properties: &'a Properties, key: &str) -> Option<&'a str> {
    match properties.get(key) {
        Some(PropertyValue::StringValue(value)) => Some(value),
        _ => None
    }
}

fn get_tile_mesh_data(tileset: &Tileset, tile_id: u32, flip_y: bool) -> TileMeshData {
    let ts = tileset;                                                       // Tileset (renamed for brevity)
    let img = ts.image.as_ref().expect("Tileset must have a single image");
    let tsm = tileset.margin as f32;                                        // Tileset margin
    let tssp = tileset.spacing as f32;                                      // Tileset spacing
    let (tiw, tih) = (ts.tile_width as f32, ts.tile_height as f32);         // Tile width / height
    let (tixi, tiyi) = (tile_id % ts.columns, tile_id / ts.columns);        // Tile x / y (ints)
    let (tix, tiy) = (tixi as f32 * tiw, tiyi as f32 * tih);                // Tile x / y (floats)
    let tss = Vec2::new(img.width as f32, img.height as f32);

    // Creates UV coords
    let tsm = Vec2::new(tsm, tsm);          // Tileset margin
    let (uv1, uv2, uv3, uv4) = if !flip_y {
        let tip = Vec2::new(tix, tiy) + Vec2::new(0.0, tih);     // Tile position
        let tisp = Vec2::new(tixi as f32, tiyi as f32) * tssp;   // Tile spacing
        let uv1 = tip + tsm + tisp;
        let uv2 = uv1 + Vec2::new(tiw, 0.0);
        let uv3 = uv1 + Vec2::new(tiw, -tih);
        let uv4 = uv1 + Vec2::new(0.0, -tih);
        (uv1, uv2, uv3, uv4)
    }
    else {
        panic!("Not yet implemented");
    };
    let (uv1, uv2, uv3, uv4) = (uv1/tss, uv2/tss, uv3/tss, uv4/tss);

    TileMeshData {
        size: Vec2::new(tiw, tih),
        uv1,
        uv2,
        uv3,
        uv4
    }
}

// A Group layer that has has been parsed (split between the terrain layers and the optional meta layers)
struct SplitGroupLayer<'map> {
    terrain_layers: Vec<TileLayer<'map>>,
    meta_layers: Vec<MetaLayer<'map>>
}

#[derive(Debug, Clone)]
pub(crate) struct ClimbingError(String);
impl std::fmt::Display for ClimbingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}


// Holds meta tiles that are either:
// 1) All geom (Tiles represent geometry, which is the shape of the terrain tiles in the graphics engine)
// 2) All coll (Tiles represent collision, which is used in the physics engine)
// 3) All geom_coll (Tiles represent geometry and collision)
enum MetaLayer<'map> {
    GeomColl(TileLayer<'map>),
    Geom(TileLayer<'map>),
    Coll(TileLayer<'map>)
}

impl<'map> MetaLayer<'map> {
    fn get_tile(&'map self, x: i32, y: i32) -> Option<MetaTile<'map>> {
        match self {
            Self::GeomColl(layer) => layer
                .get_tile(x, y)
                .map(|tile| MetaTile::GeomColl(tile.get_tile().unwrap())),
            Self::Geom(layer) => layer
                .get_tile(x, y)
                .map(|tile| MetaTile::Geom(tile.get_tile().unwrap())),
            Self::Coll(layer) => layer
                .get_tile(x, y)
                .map(|tile| MetaTile::Coll(tile.get_tile().unwrap())),
        }
    }
}

// Tile from a `MetaLayer`
enum MetaTile<'map> {
    GeomColl(Tile<'map>),
    Geom(Tile<'map>),
    Coll(Tile<'map>)
}

impl<'map> MetaTile<'map> {

    /// Geom tile type followed by coll tile type
    fn get_types(&self) -> (TileType, TileType) {
        match self {
            MetaTile::GeomColl(tile) => {
                let t_type = get_string_property(&tile.properties, "type").unwrap_or("floor");
                let t_type = TileType::from_str(t_type).unwrap();
                (t_type, t_type)
            }
            MetaTile::Geom(tile) => {
                let t_type = get_string_property(&tile.properties, "type").unwrap_or("floor");
                let t_type = TileType::from_str(t_type).unwrap();
                (t_type, TileType::Floor)
            }
            MetaTile::Coll(tile) => {
                let t_type = get_string_property(&tile.properties, "type").unwrap_or("floor");
                let t_type = TileType::from_str(t_type).unwrap();
                (TileType::Floor, t_type)
            }
        }
    }
}

// Used for "climbing" a vertical strip from a group layer.
// There should be two of these when climbing: One for determining geometry, and one for determining collision.
struct Climber {
    climb_status: ClimbStatus,
    position: Vec3,
    offset: Vec2,
    tile_height: f32
}

impl Climber {
    fn new(offset: Vec2, tile_height: f32) -> Self {
        Self {
            climb_status: ClimbStatus::NotClimbing,
            position: Vec3::new(offset.x, offset.y, 0.0),
            offset,
            tile_height
        }
    }

    /// Compares current climb status and the next tile encountered, and "climbs" appropriately.
    fn climb(
        &mut self,
        tile_type: TileType,
        tile_x: i32,
        tile_y: i32,
        group_layer_name: &str
    ) {
        self.climb_status = ClimbStatus::next(self.climb_status, tile_type, tile_x, tile_y, group_layer_name);
        if self.climb_status == ClimbStatus::NotClimbing {
            self.position.z -= self.tile_height;
        }
        else if self.climb_status.is_climbing_wall() {
            self.position.y += self.tile_height;
        }
        else if self.climb_status == ClimbStatus::FinishedClimbing {
            let ydiff = self.position.y - self.offset.y;
            self.position.y = self.offset.y;
            self.position.z -= ydiff + self.tile_height;
        }
    }

    fn geom_shape(&self, prev_status: ClimbStatus) -> Result<GeomShape, ClimbingError> {
        match self.climb_status {
            ClimbStatus::NotClimbing => match prev_status {
                ClimbStatus::ClimbingWallS | ClimbStatus::NotClimbing | ClimbStatus::FinishedClimbing => Ok(GeomShape::Floor),
                ClimbStatus::ClimbingWallSE => Ok(GeomShape::WallEndSE),
                ClimbStatus::ClimbingWallSW => Ok(GeomShape::WallEndSW)
            }
            ClimbStatus::ClimbingWallS => Ok(GeomShape::Wall),
            ClimbStatus::ClimbingWallSE => match prev_status {
                ClimbStatus::NotClimbing | ClimbStatus::FinishedClimbing => Ok(GeomShape::WallStartSE),
                ClimbStatus::ClimbingWallSE => Ok(GeomShape::WallSE),
                _ => Err(ClimbingError(format!("Entered state {:?}, after state {:?}", self.climb_status, prev_status)))
            },
            ClimbStatus::ClimbingWallSW => match prev_status {
                ClimbStatus::NotClimbing | ClimbStatus::FinishedClimbing => Ok(GeomShape::WallStartSW),
                ClimbStatus::ClimbingWallSW => Ok(GeomShape::WallSW),
                _ => Err(ClimbingError(format!("Entered state {:?}, after state {:?}", self.climb_status, prev_status)))
            },
            ClimbStatus::FinishedClimbing => match prev_status {
                ClimbStatus::FinishedClimbing => Err(ClimbingError(format!("Entered state {:?}, after state {:?}", self.climb_status, prev_status))),
                _ => Ok(GeomShape::Floor)
            }
        }
    }
}