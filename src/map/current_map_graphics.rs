use bevy::prelude::*;
use bevy::render::render_resource::VertexFormat;
use bevy::utils::HashMap;
use crate::map::{ TileGraphics, TileOrientation };

/// Tile id global to a tiled map
pub type FirstGid = u32;

#[derive(Default)]
pub struct CurrentMapGraphics {
    pub tileset_image_handles: HashMap<FirstGid, Handle<Image>>,    // Tileset id -> image
    pub chunk_width: u32,                                           // Width in tiles
    pub chunk_height: u32,                                          // Height in tiles
    pub tile_width: u32,                                            // Width in pixels
    pub tile_height: u32,                                           // Height in pixels
    pub chunk_data: HashMap<ChunkKey, Chunk>                        // Chunked mesh data
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ChunkKey {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub tileset: FirstGid
}

pub struct Chunk {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>
}

impl Chunk {
    fn add_tile(graphics: TileGraphics) {
        match graphics.orientation {
            TileOrientation::Floor => {

            },
            TileOrientation::Wall => {

            },
            _ => {
                panic!("Unsupported tile orientation '{:?}'", graphics.orientation);
            }
        }
    }
}