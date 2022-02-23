use crate::map::TileGraphics;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LoadMapEvent(pub String);

#[derive(Debug, Clone, PartialEq)]
pub struct AddTileGraphicsEvent(pub TileGraphics);

#[derive(Debug, Clone, PartialEq)]
pub struct SpawnMapEntitiesEvent;