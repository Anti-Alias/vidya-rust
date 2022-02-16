use bevy::prelude::*;
use tiled::*;

use crate::map::{GeomShape, TileType, AddTileGraphicsEvent, TileGraphics};


// Fire events that cause map to populate
pub(crate) fn add_tiles_from_map(
    tiled_map: &tiled::Map,
    mut graphics_events: EventWriter<AddTileGraphicsEvent>,
) {
    // For all group layers in the root...
    for root_layer in tiled_map.layers() {
        match &root_layer.layer_type() {
            LayerType::GroupLayer(group_layer) => {

                // Split the sub layers between terrain and meta layers
                let SplitGroupLayer {
                    terrain_layers,
                    meta_layers
                } = split_group_layer(group_layer);


                // Populate tiles from group layer
                log::debug!("Processing group layer {}", &root_layer.data().name);
                let offset = Vec2::new(
                    root_layer.data().offset_x,
                    root_layer.data().offset_y
                );
                add_tiles_from_group_layer(
                    &meta_layers,
                    &terrain_layers,
                    offset,
                    tiled_map,
                    &mut graphics_events
                );
            },
            _ => panic!("All root layers must be group layers")
        }
    }
}


fn add_tiles_from_group_layer(
    m_layers: &[MetaLayer],                                 // Group meta layers
    t_layers: &[TileLayer],                                 // Group terrain layers
    offset: Vec2,                                           // Group offset
    map: &Map,                                              // Map itself
    graphics_events: &mut EventWriter<AddTileGraphicsEvent> // Graphics event publisher
) {
    // For all columns in the group...
    let (w, h) = (map.width as usize, map.height as usize);
    let tile_size = Vec2::new(map.tile_width as f32, map.tile_height as f32);
    for x in 0..w {

        // Make climbers at the bottom of the vertical strip...
        let mut geom_climber = Climber::new(offset, tile_size.y);
        let mut coll_climber = Climber::new(offset, tile_size.y);

        // Traverse the strip from bottom to top
        for y in (0..h).rev() {

            // Gets meta tile (optional) and terrain tiles at (x, y)
            let meta_tile = m_layers
                .iter()
                .next()
                .and_then(|gc_layer| gc_layer.get_tile(x, y));
            let t_tiles = t_layers
                .iter()
                .flat_map(|t_layer| t_layer.get_tile(x, y));

            // "Climb" the current tile at x, y
            add_tiles(
                meta_tile,
                t_tiles,
                tile_size,
                map,
                &mut geom_climber,
                &mut coll_climber,
                graphics_events
            );
        }
    }
}

fn add_tiles<'map>(
    meta_tile: Option<MetaTile<'map>>,
    terrain_tiles: impl Iterator<Item=LayerTile<'map>>,
    tile_size: Vec2,
    map: &'map Map,
    geom_climber: &mut Climber,
    coll_climber: &mut Climber,
    graphics_events: &mut EventWriter<AddTileGraphicsEvent>,
) {

    // Gets the geom/coll types of current meta tile and uses it to "climb" with both climbers
    let (geom_type, coll_type) = meta_tile
        .map(|tile| tile.get_types())
        .unwrap_or((TileType::Floor, TileType::Floor));
    geom_climber.climb(geom_type);
    coll_climber.climb(coll_type);
    let geom_shape = geom_climber.climb_status.to_geom_shape();

    // For all terrain layers belonging to the same layer group in the same position...
    let tilesets = map.tilesets();
    for t_tile in terrain_tiles {

        // Fire graphics event
        let tileset_index = tilesets
            .iter()
            .position(|tileset| &tileset.name == &t_tile.tileset.name)
            .unwrap() as u32;
        let event = AddTileGraphicsEvent(TileGraphics {
            tileset_index,
            tile_index: t_tile.id as u32,
            position: geom_climber.position,
            size: tile_size,
            shape: geom_shape
        });

        // Send event for adding tile's graphics
        log::debug!("Fired event {:?}", event);
        graphics_events.send(event);
    }
}

fn split_group_layer<'map>(group_layer: &'map GroupLayer<'map>) -> SplitGroupLayer<'map>{

    // Goes through sub layers and splits them
    let mut terrain_layers = Vec::new();
    let mut meta_layers = Vec::new();
    for sub_layer in group_layer.layers() {
        let sub_properties = &sub_layer.data().properties;
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

/// Determines the status of a climb.
/// Used in conjunction with `Climber`
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
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
    fn next(prev_status: ClimbStatus, tile_type: TileType) -> Self {
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
                panic!("Encountered a {:?} tile while in climb status {:?}", tile_type, prev_status)
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
                panic!("Encountered a {:?} tile while in climb status {:?}", tile_type, prev_status)
            }
            Self::ClimbingWallSE
        }
        else if tile_type == TileType::WallStartSW {
            let is_status_valid =
                prev_status == Self::NotClimbing ||
                prev_status == Self::FinishedClimbing;
            if !is_status_valid {
                panic!("Encountered a {:?} tile while in climb status {:?}", tile_type, prev_status)
            }
            Self::ClimbingWallSW
        }
        else if tile_type == TileType::WallEndSE {
            if prev_status != Self::ClimbingWallSE {
                panic!("Encountered a {:?} tile while in climb status {:?}", tile_type, prev_status)
            }
            Self::NotClimbing
        }
        else if tile_type == TileType::WallEndSW {
            if prev_status != Self::ClimbingWallSW {
                panic!("Encountered a {:?} tile while in climb status {:?}", tile_type, prev_status)
            }
            Self::NotClimbing
        }
        else if tile_type.is_lip() {
            if !(prev_status.is_climbing_wall() || prev_status == Self::NotClimbing) {
                panic!("Encountered a {:?} tile while in climb status {:?}", tile_type, prev_status)
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

    fn to_geom_shape(self) -> GeomShape {
        match self {
            Self::NotClimbing => GeomShape::Floor,
            Self::FinishedClimbing => GeomShape::Floor,
            Self::ClimbingWallS => GeomShape::Wall,
            Self::ClimbingWallSE => GeomShape::WallSE,
            Self::ClimbingWallSW => GeomShape::WallSW
        }
    }
}

// Helper function that assumes a property is a string
fn get_string_property<'a>(properties: &'a Properties, key: &str) -> Option<&'a str> {
    match properties.get(key) {
        Some(PropertyValue::StringValue(value)) => Some(value),
        _ => None
    }
}

// A Group layer that has has been parsed (split between the terrain layers and the optional meta layers)
struct SplitGroupLayer<'map> {
    terrain_layers: Vec<TileLayer<'map>>,
    meta_layers: Vec<MetaLayer<'map>>
}

// Holds meta tiles that are either:
// 1) All geom (Tiles represent geometry, which is the shape of the terrain tiles in the graphics engine)
// 2) All coll (Tiles represent collision, which is used in the physics engine)
// 3) All geom_coll (Tiles represent geometry and collision)
#[derive(Clone)]
enum MetaLayer<'map> {
    GeomColl(TileLayer<'map>),
    Geom(TileLayer<'map>),
    Coll(TileLayer<'map>)
}

impl<'map> MetaLayer<'map> {
    fn get_tile(&'map self, x: usize, y: usize) -> Option<MetaTile<'map>> {
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
    GeomColl(&'map Tile),
    Geom(&'map Tile),
    Coll(&'map Tile)
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
    fn climb(&mut self, tile_type: TileType) {
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
        self.climb_status = ClimbStatus::next(self.climb_status, tile_type);
    }
}