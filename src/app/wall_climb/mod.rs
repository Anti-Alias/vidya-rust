use bevy::prelude::*;
use tiled::*;

use crate::map::{PrimitiveShape, TileType, AddTileGraphicsEvent, TileGraphics};

// A Group layer that has has been parsed (split between the terrain layers and the optional geom/coll/geom_coll layers)
pub(crate) struct SplitGroupLayer<'map> {
    pub terrain_layers: Vec<TileLayer<'map>>,
    pub geom_coll_layers: Vec<GeomCollLayer<'map>>
}

// Holds either geom tiles, collision tiles or both
#[derive(Clone)]
pub(crate) enum GeomCollLayer<'map> {
    GeomColl(TileLayer<'map>),
    Geom(TileLayer<'map>),
    Coll(TileLayer<'map>)
}

impl<'map> GeomCollLayer<'map> {
    pub fn get_tile(&'map self, x: usize, y: usize) -> Option<GeomCollTile<'map>> {
        match self {
            Self::GeomColl(layer) => layer
                .get_tile(x, y)
                .map(|tile| GeomCollTile::GeomColl(tile.get_tile().unwrap())),
            Self::Geom(layer) => layer
                .get_tile(x, y)
                .map(|tile| GeomCollTile::Geom(tile.get_tile().unwrap())),
            Self::Coll(layer) => layer
                .get_tile(x, y)
                .map(|tile| GeomCollTile::Coll(tile.get_tile().unwrap())),
        }
    }
}

pub(crate) enum GeomCollTile<'map> {
    GeomColl(&'map Tile),
    Geom(&'map Tile),
    Coll(&'map Tile)
}



pub(crate) fn split_group_layer<'map>(group_layer: &'map GroupLayer<'map>) -> SplitGroupLayer<'map>{

    // Allocates
    let mut terrain_layers = Vec::new();
    let mut geom_coll_layers = Vec::new();

    // Goes through sub layers and splits them
    for sub_layer in group_layer.layers() {
        let sub_properties = &sub_layer.data().properties;
        match sub_layer.layer_type() {
            LayerType::TileLayer(sub_layer) => {
                let tile_layer_type = get_string_property(sub_properties, "type").unwrap_or("terrain");
                match tile_layer_type {
                    "terrain" => terrain_layers.push(sub_layer),
                    "geom_coll" => geom_coll_layers.push(GeomCollLayer::GeomColl(sub_layer)),
                    "geom" => geom_coll_layers.push(GeomCollLayer::Geom(sub_layer)),
                    "coll" => geom_coll_layers.push(GeomCollLayer::Coll(sub_layer)),
                    _ => panic!("Unexpected tile layer type '{}'", tile_layer_type)
                }
            },
            _ => { panic!("Sub layer must be a tile layer") }
        }
    }

    // Returns split data
    SplitGroupLayer {
        terrain_layers,
        geom_coll_layers
    }
}


pub(crate) fn add_tiles_from_group_layer(
    map: &Map,                                              // Map itself
    gc_layers: &[GeomCollLayer],                            // Collision layer of group
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

            // Get the first geom_coll tile found in group
            let geom_coll = gc_layers
                .iter()
                .next()
                .and_then(|gc_layer| gc_layer.get_tile(x, y));

            // Get geom/terrain tiles from group at x, y
            let geom_tile = geom_layer
                .get_tile(x, y)
                .and_then(|tile| tile.get_tile());
            let t_tiles = t_layers
                .iter()
                .flat_map(|t_layer| t_layer.get_tile(x, y));

            // "Climb" the current tile at x, y
            log::debug!("x, y: {}, {}", x, y);
            log::debug!("Pos: {:?}", current_pos);
            let prev_status = climb_status;
            climb_status = add_tiles_using_geom_tile(
                map,
                geom_tile,
                t_tiles,
                current_pos,
                tile_size,
                prev_status,
                graphics_events
            );
            log::debug!("Cur status: {:?}", climb_status);
        }
        
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

pub(crate) fn add_tiles_using_geom_tile<'map>(
    map: &'map Map,
    g_tile: Option<&'map Tile>,
    t_tiles: impl Iterator<Item=LayerTile<'map>>,
    position: Vec3,
    size: Vec2,
    prev_climb_status: ClimbStatus,
    graphics_events: &mut EventWriter<AddTileGraphicsEvent>,
) -> ClimbStatus {

    // Gets shape of geom tile
    let tile_type = g_tile.map(|tile| get_tile_type(tile)).unwrap_or(TileType::Floor);

    // Determines next climb status based on shape and the previous climb status
    let next_status = ClimbStatus::next(prev_climb_status, tile_type);
    let shape = next_status.to_primitive_shape();

    // For all terrain layers belonging to the same layer group in the same position...
    let tilesets = map.tilesets();
    for t_tile in t_tiles {
        // Fire graphics event
        let tileset_index = tilesets
            .iter()
            .position(|tileset| &tileset.name == &t_tile.tileset.name)
            .unwrap() as u32;
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
pub(crate) enum ClimbStatus {
    NotClimbing,
    ClimbingWallS,
    ClimbingWallSE,
    ClimbingWallSW,
    FinishedClimbing
}

impl ClimbStatus {
    
    fn next(prev_status: ClimbStatus, tile_type: TileType) -> Self {
        log::debug!("Type: {:?}, prev stat: {:?}", tile_type, prev_status);
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

    fn to_primitive_shape(self) -> PrimitiveShape {
        match self {
            Self::NotClimbing => PrimitiveShape::Floor,
            Self::FinishedClimbing => PrimitiveShape::Floor,
            Self::ClimbingWallS => PrimitiveShape::Wall,
            Self::ClimbingWallSE => PrimitiveShape::WallSE,
            Self::ClimbingWallSW => PrimitiveShape::WallSW
        }
    }
}

fn get_string_property<'a>(properties: &'a Properties, key: &str) -> Option<&'a str> {
    match properties.get(key) {
        Some(PropertyValue::StringValue(value)) => Some(value),
        _ => None
    }
}

fn get_tile_type(tile: &Tile) -> TileType {
    let shape_str = get_string_property(&tile.properties, "type").unwrap_or("floor");
    TileType::from_str(shape_str).unwrap()
}