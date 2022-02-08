use bevy::math::Vec3;

/// Tile id local to a tileset
pub type LocalId = u32;

/// Information about a tile's graphics
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TileGraphics {
    pub tileset_id: u32,
    pub tile_id: u32,
    pub position: Vec3,
    pub orientation: TileOrientation
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TileOrientation {
    Floor,
    Wall,
    WallStartSE,
    WallStartSW,
    WallSE,
    WallSW,
    WallEndSE,
    WallEndSW,
    SlopeS,
}

/// Event for adding tile graphics
pub struct AddTileGraphicsEvent(TileGraphics);