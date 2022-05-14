use bevy::prelude::*;

/// Tile id local to a tileset
pub type LocalId = u32;

/// Information about a tile's graphics (position, size, uvs, shape)
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TileGraphics {
    pub tileset_index: u32,
    pub translation: Vec3,
    pub mesh_data: TileMeshData,
    pub shape: TileShape
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TileMeshData {
    pub size: Vec2,
    pub uv1: Vec2,
    pub uv2: Vec2,
    pub uv3: Vec2,
    pub uv4: Vec2
}

/// Type of meta tile this is.
/// Maps directly to what is is in a map file.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TileType {
    Floor,
    Wall,
    WallStartSE,
    WallEndSE,
    WallStartSW,
    WallEndSW,
    LipN,
    LipNE,
    LipNW,
    Slope,
    SlopeStartE,
    SlopeEndE,
    SlopeStartW,
    SlopeEndW
}

impl TileType {
    pub fn from_str(str: &str) -> Option<Self> {
        match str {
            "floor" => Some(Self::Floor),
            "wall" => Some(Self::Wall),
            "wall-start-se" => Some(Self::WallStartSE),
            "wall-end-se" => Some(Self::WallEndSE),
            "wall-start-sw" => Some(Self::WallStartSW),
            "wall-end-sw" => Some(Self::WallEndSW),
            "lip-n" => Some(Self::LipN),
            "lip-ne" => Some(Self::LipNE),
            "lip-nw" => Some(Self::LipNW),
            "slope" => Some(Self::Slope),
            "slope-start-e" => Some(Self::SlopeStartE),
            "slope-end-e" => Some(Self::SlopeEndE),
            "slope-start-w" => Some(Self::SlopeStartW),
            "slope-end-w" => Some(Self::SlopeEndW),
            _ => None
        }
    }

    pub fn is_lip(self) -> bool {
        self == Self::LipN ||
        self == Self::LipNE ||
        self == Self::LipNW
    }
}

/// Represents the immediate 3D shape of a meta tile, not to be confused with [`TileType`].
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TileShape {
    Floor,
    Wall,
    WallStartSE,
    WallSE,
    WallEndSE,
    WallStartSW,
    WallSW,
    WallEndSW,
    SlopeS,
    SlopeStartE,
    SlopeE,
    SlopeEndE,
    SlopeStartW,
    SlopeW,
    SlopeEndW
}
