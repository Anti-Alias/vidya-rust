use crate::map::VidyaMap;

use bevy::prelude::*;


// Staging resource for a map that is being loaded
#[derive(Debug)]
pub struct CurrentMap {
    pub file: String,                   // Name of the file the map came from
    pub map_handle: Handle<VidyaMap>,   // Map handle
    pub map_entity: Entity              // Parent entity of map's chunks
}