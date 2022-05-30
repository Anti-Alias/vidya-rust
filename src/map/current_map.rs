use crate::map::VidyaMap;
use crate::physics::{Terrain, TerrainPiece, Coords};

use bevy::prelude::*;


// Temporary staging resource for a map's collision data / metadata.
#[derive(Clone)]
pub struct CurrentMap {
    pub name: String,
    pub map_handle: Handle<VidyaMap>,   // Map handle
    pub terrain: Terrain                // Terrain of the current map
}

impl CurrentMap {

    /// Sets the terrain piece at the specified coordinates
    pub fn set_terrain_piece(&mut self, piece: TerrainPiece, coords: Coords) {
        let current_piece_ref = self.terrain.get_or_create_mut(coords);
        *current_piece_ref = piece;
    }
}