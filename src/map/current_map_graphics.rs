use std::f32::consts::SQRT_2;

use bevy::{prelude::*, asset::LoadState};
use bevy::utils::HashMap;

use crate::map::{ TileGraphics, TileShape, TileMeshData };

// Temporary staging resource for a map's graphics data.
#[derive(Default)]
pub struct CurrentMapGraphics {
    pub tileset_image_handles: Vec<Option<Handle<Image>>>,  // Tileset name -> image
    pub chunk_size: Vec3,                                   // Width, height and depth of chunks
    pub chunks: HashMap<ChunkKey, Chunk>                    // Chunked mesh data
}

impl CurrentMapGraphics {
    pub fn new(chunk_size: Vec3) -> Self {
        Self {
            chunk_size,
            ..Default::default()
        }
    }

    pub fn get_load_state(&self, assets: &AssetServer) -> LoadState {
        let handle_ids = self
            .tileset_image_handles
            .iter()
            .flatten()
            .map(|handle| { handle.id });
        assets.get_group_load_state(handle_ids)
    }

    pub fn add_tile(&mut self, tile: TileGraphics) {
        let chunk_size = self.chunk_size;
        let tile_pos = tile.translation / chunk_size;
        let (x, y, z) = (tile_pos.x as i32, tile_pos.y as i32, tile_pos.z as i32);
        let tileset_index = tile.tileset_index as usize;
        let key = ChunkKey { x, y, z, tileset_index };
        let chunk = self.chunks.entry(key).or_default();
        chunk.add_tile(tile);
        log::trace!("Added tile {:?} at pos {:?} to {:?}", tile.shape, tile.translation, key);
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
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>
}

impl Chunk {
    fn add_tile(&mut self, tile: TileGraphics) {

        const X: usize = 0;
        const Y: usize = 1;
        const Z: usize = 2;

        // Gets buffers to write to (positions, normal, uvs, indices)
        let p = &mut self.positions;
        let n = &mut self.normals;
        let uvs = &mut self.uvs;
        let i = &mut self.indices;

        // Gets tile's postition and pushes it closer to the camera
        let mut tp = tile.translation.to_array();

        // Gets tile's mesh data
        let md = tile.mesh_data;
        let (tw, th) = (md.size.x, md.size.y);
        let vlen = p.len() as u32;

        // Writes tile to buffers
        match tile.shape {
            TileShape::Floor => {
                // Positions (4)
                p.push(tp);
                tp[X] += tw;
                p.push(tp);
                tp[Z] -= th;
                p.push(tp);
                tp[X] -= tw;
                p.push(tp);
                // Normals (4)
                let norm = [0.0, 1.0, 0.0];
                for _ in 0..4 { n.push(norm); }
                // UVs and indices
                push_uv_indices_4(&md, uvs, i, vlen);
            },
            TileShape::Wall => {
                // Positions (4)
                p.push(tp);
                tp[X] += tw;
                p.push(tp);
                tp[Y] += th;
                p.push(tp);
                tp[X] -= tw;
                p.push(tp);
                // Normals (4)
                let norm = [0.0, 0.0, 1.0];
                for _ in 0..4 { n.push(norm); }
                // UVs and indices
                push_uv_indices_4(&md, uvs, i, vlen);
            },
            TileShape::WallStartSE => {
                // Vertices (6)
                p.push(tp);
                tp[X] += tw;
                p.push(tp);
                tp[Z] -= th;
                p.push(tp);
                p.push(tp);
                tp[X] -= tw;
                tp[Y] += th;
                tp[Z] += th;
                p.push(tp);
                tp[Y] -= th;
                p.push(tp);

                // Normals (6)
                let up = [0.0, 1.0, 0.0];
                let se = [1.0/SQRT_2, 0.0, 1.0/SQRT_2];
                n.push(up);
                n.push(up);
                n.push(up);
                n.push(se);
                n.push(se);
                n.push(se);

                // UVs and indices
                uvs.push(md.uv1.to_array());
                uvs.push(md.uv2.to_array());
                uvs.push(md.uv3.to_array());
                uvs.push(md.uv3.to_array());
                uvs.push(md.uv4.to_array());
                uvs.push(md.uv1.to_array());

                // Indices
                i.push(vlen);
                i.push(vlen+1);
                i.push(vlen+2);
                i.push(vlen+3);
                i.push(vlen+4);
                i.push(vlen+5);
            }
            TileShape::WallStartSW => {
                // Vertices (6)
                tp[Z] -= th;
                p.push(tp);
                tp[Z] += th;
                p.push(tp);
                tp[X] += tw;
                p.push(tp);
                p.push(tp);
                tp[Y] += th;
                p.push(tp);
                tp[X] -= tw;
                tp[Y] -= th;
                tp[Z] -= th;
                p.push(tp);

                // Normals (6)
                let up = [0.0, 1.0, 0.0];
                let sw = [-1.0/SQRT_2, 0.0, 1.0/SQRT_2];
                n.push(up);
                n.push(up);
                n.push(up);
                n.push(sw);
                n.push(sw);
                n.push(sw);

                // UVs
                uvs.push(md.uv4.to_array());
                uvs.push(md.uv1.to_array());
                uvs.push(md.uv2.to_array());
                uvs.push(md.uv2.to_array());
                uvs.push(md.uv3.to_array());
                uvs.push(md.uv4.to_array());

                // Indices
                i.push(vlen);
                i.push(vlen+1);
                i.push(vlen+2);
                i.push(vlen+3);
                i.push(vlen+4);
                i.push(vlen+5);
            }
            TileShape::WallSE => {
                // Vertices (4)
                p.push(tp);
                tp[X] += tw;
                tp[Y] -= th;
                tp[Z] -= th;
                p.push(tp);
                tp[Y] += th;
                p.push(tp);
                tp[X] -= tw;
                tp[Y] += th;
                tp[Z] += th;
                p.push(tp);
                // Normals (4)
                let norm = [1.0/SQRT_2, 0.0, 1.0/SQRT_2];
                for _ in 0..4 {
                    n.push(norm);
                }
                // UVs and indices
                push_uv_indices_4(&md, uvs, i, vlen);
            }
            TileShape::WallSW => {
                // Vertices (4)
                tp[Y] -= th;
                tp[Z] -= th;
                p.push(tp);
                tp[X] += tw;
                tp[Y] += th;
                tp[Z] += th;
                p.push(tp);
                tp[Y] += th;
                p.push(tp);
                tp[X] -= tw;
                tp[Y] -= th;
                tp[Z] -= th;
                p.push(tp);
                // Normals (4)
                let norm = [-1.0/SQRT_2, 0.0, 1.0/SQRT_2];
                for _ in 0..4 {
                    n.push(norm);
                }
                // UVs and indices
                push_uv_indices_4(&md, uvs, i, vlen);
            }
            TileShape::WallEndSE => {
                // Vertices (6)
                p.push(tp);
                tp[X] += tw;
                tp[Y] -= th;
                tp[Z] -= th;
                p.push(tp);
                tp[Y] += th;
                p.push(tp);
                p.push(tp);
                tp[X] -= tw;
                p.push(tp);
                tp[Z] += th;
                p.push(tp);

                // Normals (6)
                let up = [0.0, 1.0, 0.0];
                let se = [1.0/SQRT_2, 0.0, 1.0/SQRT_2];
                n.push(se);
                n.push(se);
                n.push(se);
                n.push(up);
                n.push(up);
                n.push(up);

                // UVs
                uvs.push(md.uv1.to_array());
                uvs.push(md.uv2.to_array());
                uvs.push(md.uv3.to_array());
                uvs.push(md.uv3.to_array());
                uvs.push(md.uv4.to_array());
                uvs.push(md.uv1.to_array());

                // Indices
                i.push(vlen);
                i.push(vlen+1);
                i.push(vlen+2);
                i.push(vlen+3);
                i.push(vlen+4);
                i.push(vlen+5);
            }
            TileShape::WallEndSW => {
                // Vertices (6)
                tp[Z] -= th;
                p.push(tp);
                tp[Y] -= th;
                p.push(tp);
                tp[X] += tw;
                tp[Y] += th;
                tp[Z] += th;
                p.push(tp);
                p.push(tp);
                tp[Z] -= th;
                p.push(tp);
                tp[X] -= th;
                p.push(tp);

                // Normals (6)
                let up = [0.0, 1.0, 0.0];
                let sw = [-1.0/SQRT_2, 0.0, 1.0/SQRT_2];
                n.push(sw);
                n.push(sw);
                n.push(sw);
                n.push(up);
                n.push(up);
                n.push(up);

                // UVs
                uvs.push(md.uv4.to_array());
                uvs.push(md.uv1.to_array());
                uvs.push(md.uv2.to_array());
                uvs.push(md.uv2.to_array());
                uvs.push(md.uv3.to_array());
                uvs.push(md.uv4.to_array());

                // Indices
                i.push(vlen);
                i.push(vlen+1);
                i.push(vlen+2);
                i.push(vlen+3);
                i.push(vlen+4);
                i.push(vlen+5);
            }
            _ => {
                //panic!("Unsupported tile shape '{:?}'", tile.shape);
            }
        }
    }
}

// Pushes 4 uv values and 6 indices (4 vertices)
fn push_uv_indices_4(
    mesh_data: &TileMeshData,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    vlen: u32
) {
    uvs.push(mesh_data.uv1.to_array());
    uvs.push(mesh_data.uv2.to_array());
    uvs.push(mesh_data.uv3.to_array());
    uvs.push(mesh_data.uv4.to_array());

    indices.push(vlen);
    indices.push(vlen+1);
    indices.push(vlen+2);
    indices.push(vlen+2);
    indices.push(vlen+3);
    indices.push(vlen);
}