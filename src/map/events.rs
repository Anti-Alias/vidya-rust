use super::climb::TileInfo;

#[derive(Debug, Clone)]
pub struct LoadMapEvent(pub String);

#[derive(Debug, Copy, Clone)]
pub struct AddTileEvent(pub TileInfo);

#[derive(Debug, Clone)]
pub struct SpawnMapEntitiesEvent;