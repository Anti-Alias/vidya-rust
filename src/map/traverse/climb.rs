use bevy::prelude::*;

use crate::map::{TileType, TileShape};
use crate::physics::{Coords};

// Used for "climbing" a vertical strip from a group layer.
// There should be two of these when climbing: One for determining geometry, and one for determining collision.
pub(crate) struct Climber {
    climb_status: ClimbStatus,
    prev_status: ClimbStatus,
    position: Vec3,
    next_position: Vec3,
    offset: Vec2,
    tile_size: Vec3
}

impl Climber {
    pub fn new(offset: Vec2, tile_size: Vec3) -> Self {
        let position = Vec3::new(offset.x, offset.y, 0.0);
        Self {
            climb_status: ClimbStatus::NotClimbing,
            prev_status: ClimbStatus::NotClimbing,
            position,
            next_position: position,
            offset,
            tile_size
        }
    }

    /// Position of this climber
    pub fn position(&self) -> Vec3 { self.position }

    /// Coordinates of current tile for collision
    pub fn coords(&self) -> Coords {
        let c = self.position / self.tile_size;
        Coords::new(
            c.x as i32,
            c.y as i32,
            c.z as i32
        )
    }

    /// Coordinates of next tile for collision
    pub fn next_coords(&self) -> Coords {
        let c = self.next_position / self.tile_size;
        Coords::new(
            c.x as i32,
            c.y as i32,
            c.z as i32
        )
    }

    /// Compares current climb status and the next tile encountered, and "climbs" appropriately.
    pub fn climb(
        &mut self,
        tile_type: TileType,
        tile_x: i32,
        tile_y: i32,
        group_layer_name: &str
    ) -> Result<(), ClimbingError> {
        let position = self.next_position;
        let prev_status = self.climb_status;
        self.climb_status = ClimbStatus::next(self.climb_status, tile_type, tile_x, tile_y, group_layer_name)?;
        if self.climb_status == ClimbStatus::NotClimbing {
            self.next_position.z -= self.tile_size.y;
        }
        else if self.climb_status.is_climbing_wall() {
            self.next_position.y += self.tile_size.y;
        }
        else if self.climb_status == ClimbStatus::FinishedClimbing {
            let ydiff = self.next_position.y - self.offset.y;
            self.next_position.y = self.offset.y;
            self.next_position.z -= ydiff + self.tile_size.y;
        }
        self.position = position;
        self.prev_status = prev_status;
        Ok(())
    }

    pub fn climb_status(&self) -> ClimbStatus { self.climb_status }

    pub fn tile_shape(&self) -> Result<TileShape, ClimbingError> {
        match self.climb_status {
            ClimbStatus::NotClimbing => match self.prev_status {
                ClimbStatus::ClimbingWallS | ClimbStatus::NotClimbing | ClimbStatus::FinishedClimbing => Ok(TileShape::Floor),
                ClimbStatus::ClimbingWallSE => Ok(TileShape::WallEndSE),
                ClimbStatus::ClimbingWallSW => Ok(TileShape::WallEndSW)
            }
            ClimbStatus::ClimbingWallS => Ok(TileShape::Wall),
            ClimbStatus::ClimbingWallSE => match self.prev_status {
                ClimbStatus::NotClimbing | ClimbStatus::FinishedClimbing => Ok(TileShape::WallStartSE),
                ClimbStatus::ClimbingWallSE => Ok(TileShape::WallSE),
                _ => Err(ClimbingError(format!("Entered state {:?}, after state {:?}", self.climb_status, self.prev_status)))
            },
            ClimbStatus::ClimbingWallSW => match self.prev_status {
                ClimbStatus::NotClimbing | ClimbStatus::FinishedClimbing => Ok(TileShape::WallStartSW),
                ClimbStatus::ClimbingWallSW => Ok(TileShape::WallSW),
                _ => Err(ClimbingError(format!("Entered state {:?}, after state {:?}", self.climb_status, self.prev_status)))
            },
            ClimbStatus::FinishedClimbing => match self.prev_status {
                ClimbStatus::FinishedClimbing => Err(ClimbingError(format!("Entered state {:?}, after state {:?}", self.climb_status, self.prev_status))),
                _ => Ok(TileShape::Floor)
            }
        }
    }
}

/// Error encountered while climbing
#[derive(Debug, Clone)]
pub(crate) struct ClimbingError(pub String);
impl std::fmt::Display for ClimbingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

/// Determines the status of a climb.
/// Used in conjunction with `Climber`
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum ClimbStatus {
    /// Tile is treated as a floor. Next tile is 1 farther (z)
    NotClimbing,
    // Tile is treated as a wall. Next tile is 1 higher (y)
    ClimbingWallS,
    /// Tile is treated as a south-east wall. Next tile is 1 higher (y)
    ClimbingWallSE,
    // Tile is treated as a south-west wall. Next tile is 1 higher (y)
    ClimbingWallSW,
    // Just encountered a "lip" tile. Tile is treated as floor. Next tile is N tiles below (y) and N tiles farther (z), where N represents how high we were in tiles
    FinishedClimbing
}

impl ClimbStatus {
    
    /// Takes into account the climb status of the previous tile and the type of the current tile to determine the
    /// climb status for the current tile.
    pub fn next(
        prev_status: ClimbStatus,
        tile_type: TileType,
        tile_x: i32,
        tile_y: i32,
        group_layer_name: &str
    ) -> Result<Self, ClimbingError> {
        // What should the resulting climb status be, considering the current collision tile and the previous climb status?
        // Yes, this is ugly and no, I'm not going to fix it...
        if tile_type == TileType::Floor {
            let is_status_valid =
                prev_status == Self::NotClimbing ||
                prev_status == Self::ClimbingWallS ||
                prev_status == Self::ClimbingWallSE ||
                prev_status == Self::ClimbingWallSW ||
                prev_status == Self::FinishedClimbing;
            if !is_status_valid {
                return Err(Self::make_climbing_error(tile_type, prev_status, group_layer_name, tile_x, tile_y));
            }
            Ok(Self::NotClimbing)
        }
        else if tile_type == TileType::Wall {
            if  prev_status == Self::NotClimbing ||
                prev_status == Self::ClimbingWallS ||
                prev_status == Self::FinishedClimbing {
                Ok(Self::ClimbingWallS)
            }
            else if prev_status == Self::ClimbingWallSE {
                Ok(Self::ClimbingWallSE)
            }
            else if prev_status == Self::ClimbingWallSW {
                Ok(Self::ClimbingWallSW)
            }
            else {
                // Slopes???
                todo!()
            }
        }
        else if tile_type == TileType::WallStartSE {
            let is_status_valid =
                prev_status == Self::NotClimbing ||
                prev_status == Self::FinishedClimbing;
            if !is_status_valid {
                return Err(Self::make_climbing_error(tile_type, prev_status, group_layer_name, tile_x, tile_y));
            }
            Ok(Self::ClimbingWallSE)
        }
        else if tile_type == TileType::WallStartSW {
            let is_status_valid =
                prev_status == Self::NotClimbing ||
                prev_status == Self::FinishedClimbing;
            if !is_status_valid {
                return Err(Self::make_climbing_error(tile_type, prev_status, group_layer_name, tile_x, tile_y));
            }
            Ok(Self::ClimbingWallSW)
        }
        else if tile_type == TileType::WallEndSE {
            if prev_status != Self::ClimbingWallSE {
                return Err(Self::make_climbing_error(tile_type, prev_status, group_layer_name, tile_x, tile_y));
            }
            Ok(Self::NotClimbing)
        }
        else if tile_type == TileType::WallEndSW {
            if prev_status != Self::ClimbingWallSW {
                return Err(Self::make_climbing_error(tile_type, prev_status, group_layer_name, tile_x, tile_y));
            }
            Ok(Self::NotClimbing)
        }
        else if tile_type.is_lip() {
            if !(prev_status.is_climbing_wall() || prev_status == Self::NotClimbing) {
                return Err(Self::make_climbing_error(tile_type, prev_status, group_layer_name, tile_x, tile_y));
            }
            Ok(Self::FinishedClimbing)
        }
        else {
            return Err(Self::make_climbing_error(tile_type, prev_status, group_layer_name, tile_x, tile_y));
        }
    }

    pub fn is_climbing_wall(self) -> bool {
        self == Self::ClimbingWallS ||
        self == Self::ClimbingWallSE ||
        self == Self::ClimbingWallSW
    }

    fn make_climbing_error(tile_type: TileType, prev_status: ClimbStatus, group_layer_name: &str, tile_x: i32, tile_y: i32) -> ClimbingError {
        ClimbingError(format!(
            "Encountered a {:?} tile while in climb status {:?} on group layer '{}' at {}, {}",
            tile_type,
            prev_status,
            group_layer_name,
            tile_x,
            tile_y
        ))
    }
}