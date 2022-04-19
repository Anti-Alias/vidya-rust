use crate::map::VidyaMap;
use crate::physics::{Terrain, TerrainPiece, Coords};

use bevy::prelude::*;

use super::TileShape;


// Staging resource for a map that is being loaded
pub struct CurrentMap {
    pub file: String,                   // Name of the file the map came from
    pub map_handle: Handle<VidyaMap>,   // Map handle
    pub map_entity: Entity,             // Parent entity of map's chunks
    pub terrain: Terrain                // Terrain of the current map
}

impl CurrentMap {

    /// Sets the terrain piece at the specified coordinates
    pub fn set_terrain_piece(&mut self, piece: TerrainPiece, coords: Coords) {
        let current_piece_ref = self.terrain.get_or_create_mut(coords);
        *current_piece_ref = piece
    }

    pub fn set_terrain_piece_from_shape(&mut self, tile_shape: TileShape, position: Vec3) {
        let piece_size = self.terrain.piece_size();
        let mut coords = Coords {
            x: (position.x / piece_size.x) as i32,
            y: (position.y / piece_size.y) as i32,
            z: (position.z / piece_size.z) as i32
        };
        let piece = match tile_shape {
            TileShape::Floor => {
                coords.y -= 1;
                TerrainPiece::Cuboid
            }
            TileShape::Wall => {
                TerrainPiece::Cuboid
            }
            _ => TerrainPiece::Cuboid
        };
        log::info!("Setting {:?} at coords {:?}", piece, coords);
        self.set_terrain_piece(piece, coords);
    }
}