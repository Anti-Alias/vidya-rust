use bevy::prelude::*;

use crate::map::Velocity;

/// Plugin for "Being" behavior
pub struct BeingPlugin;
impl Plugin for BeingPlugin {
    fn build(&self, app: &mut App) {

    }
}

/// Component that allows an Entity to face a direction and hold state
/// IE: Player, Creatures, etc
pub struct Being {
    /// Direction being is facing in radians
    pub direction: f32,
    /// Current high-level action that entity is doing.
    /// Used to control what behaviors an Entity can and can't be doing at any given moment
    pub state: State,
}

/// Explicit directions that can be faced
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Direction {
    N,
    S,
    E,
    W,
    NE,
    NW,
    SW,
    SE
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum State {
    Idle,
    Running,
    Jumping,
    Attacking
}