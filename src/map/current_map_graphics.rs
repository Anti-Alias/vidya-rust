use bevy::prelude::*;
use bevy::utils::HashMap;
use crate::map::{ TileGraphics, GeomShape };

/// Staging resource for the graphics of a loading tiled map
#[derive(Default)]
pub struct CurrentMapGraphics {
    pub tileset_image_handles: HashMap<String, Handle<Image>>,  // Tileset name -> image
    pub chunk_size: Vec3,                                       // Width, height and depth of chunks
    pub chunks: HashMap<ChunkKey, Chunk>                        // Chunked mesh data
}

impl CurrentMapGraphics {
    pub fn new(chunk_size: Vec3) -> Self {
        Self {
            chunk_size,
            ..Default::default()
        }
    }

    pub fn add_tile(&mut self, tile: TileGraphics) {
        let chunk_size = self.chunk_size;
        let tile_pos = tile.position / chunk_size;
        let (x, y, z) = (tile_pos.x as i32, tile_pos.y as i32, tile_pos.z as i32);
        let tileset_index = tile.tileset_index as usize;
        let key = ChunkKey { x, y, z, tileset_index };
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
    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    uvs: Vec<Vec2>,
    indices: Vec<u32>
}

impl Chunk {
    fn add_tile(&mut self, tile: TileGraphics) {
        let p = &mut self.positions;
        let n = &mut self.normals;
        let u = &mut self.uvs;
        let i = &mut self.indices;
        let ilen = i.len() as u32;

        let mut tp = tile.position;
        let ts = tile.size;
        match tile.shape {
            GeomShape::Floor => {

                // Position
                p.push(tp); tp.x += ts.x;
                p.push(tp); tp.z -= ts.y;
                p.push(tp); tp.x -= ts.x;
                p.push(tp);

                // Normals
                let up = Vec3::new(0.0, 1.0, 0.0);
                for _ in 0..4 { n.push(up); }

                // Normals
                //u.push()

                // Indices
                i.push(ilen); i.push(ilen+1); i.push(ilen+2); i.push(ilen+2); i.push(ilen+3); i.push(ilen);
            },
            GeomShape::Wall => {

            },
            _ => {
                //panic!("Unsupported tile shape '{:?}'", tile.shape);
            }
        }
    }
}