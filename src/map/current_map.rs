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
        log::info!("Setting {:?} at coords {:?}", piece, coords);
        let current_piece_ref = self.terrain.get_or_create_mut(coords);
        *current_piece_ref = piece;
    }
}