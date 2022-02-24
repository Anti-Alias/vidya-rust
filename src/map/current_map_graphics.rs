use std::f32::consts::SQRT_2;

use bevy::{ prelude::* };
use bevy::utils::HashMap;
use crate::map::{ TileGraphics, GeomShape };

use super::TileMeshData;

/// Staging resource for the graphics of a loading tiled map
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

    pub fn add_tile(&mut self, tile: TileGraphics) {
        let chunk_size = self.chunk_size;
        let tile_pos = tile.position / chunk_size;
        let (x, y, z) = (tile_pos.x as i32, tile_pos.y as i32, tile_pos.z as i32);
        let tileset_index = tile.tileset_index as usize;
        let key = ChunkKey { x, y, z, tileset_index };
        let chunk = self.chunks.entry(key).or_default();
        chunk.add_tile(tile);
        log::trace!("Added tile {:?} at pos {:?} to {:?}", tile.shape, tile.position, key);
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

        // Splits and borrows members
        let p = &mut self.positions;
        let n = &mut self.normals;
        let uvs = &mut self.uvs;
        let i = &mut self.indices;

        // Adds position, normal and 
        let mut tp = tile.position.to_array();
        let md = tile.mesh_data;
        let (tw, th) = (md.size.x, md.size.y);
        let (x, y, z) = (0, 1, 2);
        let vlen = p.len() as u32;
        match tile.shape {
            GeomShape::Floor => {
                // Positions (4)
                p.push(tp);
                tp[x] += tw;
                p.push(tp);
                tp[z] -= th;
                p.push(tp);
                tp[x] -= tw;
                p.push(tp);
                // Normals (4)
                let norm = [0.0, 1.0, 0.0];
                for _ in 0..4 { n.push(norm); }
                // UVs and indices
                push_uv_indices_4(&md, uvs, i, vlen);
            },
            GeomShape::Wall => {
                // Positions (4)
                p.push(tp);
                tp[x] += tw;
                p.push(tp);
                tp[y] += th;
                p.push(tp);
                tp[x] -= tw;
                p.push(tp);
                // Normals (4)
                let norm = [0.0, 0.0, -1.0];
                for _ in 0..4 { n.push(norm); }
                // UVs and indices
                push_uv_indices_4(&md, uvs, i, vlen);
            },
            GeomShape::WallStartSE => {
                // Vertices (6)
                p.push(tp);
                tp[x] += tw;
                p.push(tp);
                tp[z] -= th;
                p.push(tp);
                p.push(tp);
                tp[x] -= tw;
                tp[y] += th;
                tp[z] += th;
                p.push(tp);
                tp[y] -= th;
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
            GeomShape::WallStartSW => {
                // Vertices (6)
                tp[z] -= th;
                p.push(tp);
                tp[z] += th;
                p.push(tp);
                tp[x] += tw;
                p.push(tp);
                p.push(tp);
                tp[y] += th;
                p.push(tp);
                tp[x] -= tw;
                tp[y] -= th;
                tp[z] -= th;
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
            GeomShape::WallSE => {
                // Vertices (4)
                p.push(tp);
                tp[x] += tw;
                tp[y] -= th;
                tp[z] -= th;
                p.push(tp);
                tp[y] += th;
                p.push(tp);
                tp[x] -= tw;
                tp[y] += th;
                tp[z] += th;
                p.push(tp);
                // Normals (4)
                let norm = [1.0/SQRT_2, 0.0, 1.0/SQRT_2];
                for _ in 0..4 {
                    n.push(norm);
                }
                // UVs and indices
                push_uv_indices_4(&md, uvs, i, vlen);
            }
            GeomShape::WallSW => {
                // Vertices (4)
                tp[y] -= th;
                tp[z] -= th;
                p.push(tp);
                tp[x] += tw;
                tp[y] += th;
                tp[z] += th;
                p.push(tp);
                tp[y] += th;
                p.push(tp);
                tp[x] -= tw;
                tp[y] -= th;
                tp[z] -= th;
                p.push(tp);
                // Normals (4)
                let norm = [-1.0/SQRT_2, 0.0, 1.0/SQRT_2];
                for _ in 0..4 {
                    n.push(norm);
                }
                // UVs and indices
                push_uv_indices_4(&md, uvs, i, vlen);
            }
            GeomShape::WallEndSE => {
                // Vertices (6)
                p.push(tp);
                tp[x] += tw;
                tp[y] -= th;
                tp[z] -= th;
                p.push(tp);
                tp[y] += th;
                p.push(tp);
                p.push(tp);
                tp[x] -= tw;
                p.push(tp);
                tp[z] += th;
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
            GeomShape::WallEndSW => {
                // Vertices (6)
                tp[z] -= th;
                p.push(tp);
                tp[y] -= th;
                p.push(tp);
                tp[x] += tw;
                tp[y] += th;
                tp[z] += th;
                p.push(tp);
                p.push(tp);
                tp[z] -= th;
                p.push(tp);
                tp[x] -= th;
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

// Pushes 6 uv values and 6 indices (6 vertices where positions and uvs are assumed to be duplicates at 2,3 and 0,5)
fn push_uv_indices_6(
    mesh_data: &TileMeshData,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    vlen: u32
) {
    uvs.push(mesh_data.uv1.to_array());
    uvs.push(mesh_data.uv2.to_array());
    uvs.push(mesh_data.uv3.to_array());
    uvs.push(mesh_data.uv3.to_array());
    uvs.push(mesh_data.uv4.to_array());
    uvs.push(mesh_data.uv1.to_array());

    indices.push(vlen);
    indices.push(vlen+1);
    indices.push(vlen+2);
    indices.push(vlen+3);
    indices.push(vlen+4);
    indices.push(vlen+5);
}