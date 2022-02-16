use bevy::prelude::*;
use bevy::utils::HashMap;
use crate::map::{ TileGraphics, GeomShape };

const CHUNK_DIM: f32 = 16.0*8.0;

#[derive(Default)]
pub struct CurrentMapGraphics {
    pub tileset_image_handles: HashMap<String, Handle<Image>>,  // Tileset name -> image
    pub chunk_size: Vec3,                                       // Width, height and depth of chunks
    pub chunks: HashMap<ChunkKey, Chunk>                        // Chunked mesh data
}

impl CurrentMapGraphics {
    pub fn new(chunk_width: f32, chunk_height: f32) -> Self {
        Self {
            chunk_size: Vec3::new(CHUNK_DIM, CHUNK_DIM, CHUNK_DIM),
            ..Default::default()
        }
    }

    pub fn add_tile(&mut self, tile: TileGraphics) {
        let chunk_size = self.chunk_size;
        let tile_pos = tile.position / chunk_size;
        let (x, y, z) = (tile_pos.x as i32, tile_pos.y as i32, tile_pos.z as i32);
        let tileset_index = tile.tileset_index;
        let key = ChunkKey {
            x,
            y,
            z,
            tileset_index: tileset_index as usize
        };
        let chunk = self.chunks.entry(key).or_default();
        chunk.add_tile(tile);
        log::debug!("Added tile {:?} at pos {:?} to {:?}", tile.shape, tile.position, key);
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ChunkKey {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub tileset_index: usize
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Chunk {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>
}

impl Chunk {
    fn add_tile(&mut self, tile: TileGraphics) {
        match tile.shape {
            GeomShape::Floor => {
                
            },
            GeomShape::Wall => {

            },
            _ => {
                //panic!("Unsupported tile shape '{:?}'", tile.shape);
            }
        }
    }
}