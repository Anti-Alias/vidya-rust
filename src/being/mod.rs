use bevy::prelude::*;

use crate::app::{AppState, AppLabel};
use crate::physics::{Velocity, Friction};

/// Plugin for "Being" behavior
pub struct BeingPlugin;
impl Plugin for BeingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(SystemSet::on_update(AppState::AppRunning)
                .with_system(update_platformers.label(AppLabel::Logic))
            )
        ;
    }
}

/// Component that allows an Entity to face a direction and hold state
/// IE: Player, Creatures, etc
#[derive(Component, Default)]
pub struct Being {
    /// Direction being is facing in radians
    pub direction: f32,
    /// Current high-level action that entity is doing.
    /// Used to control what behaviors an Entity can and can't be doing at any given moment
    pub state: State,
}

impl Being {
    pub fn direction(&self) -> Direction {
        let pi = std::f32::consts::PI;
        let pi2 = pi*2.0;
        let slice = pi2/8.0;
        let halfslice = slice / 2.0;
        let direction = ((self.direction % pi2) + pi2) % pi + halfslice;
        if direction < slice*1.0 {
            Direction::E
        }
        else if direction < slice*2.0 {
            Direction::NE
        }
        else if direction < slice*3.0 {
            Direction::N
        }
        else if direction < slice*4.0 {
            Direction::NW
        }
        else if direction < slice*5.0 {
            Direction::W
        }
        else if direction < slice*6.0 {
            Direction::SE
        }
        else if direction > slice*7.0 {
            Direction::SE
        }
        else {
            Direction::E
        }
    }

    pub fn to_cardinal_direction(&self) -> CardinalDirection {
        let pi = std::f32::consts::PI;
        let pi2 = pi*2.0;
        let slice = pi2/4.0;
        let halfslice = slice / 2.0;
        let direction = ((self.direction % pi2) + pi2) % pi + halfslice;
        if direction < slice*1.0 {
            CardinalDirection::E
        }
        else if direction < slice*2.0 {
            CardinalDirection::N
        }
        else if direction < slice*3.0 {
            CardinalDirection::W
        }
        else if direction < slice*4.0 {
            CardinalDirection::S
        }
        else {
            CardinalDirection::E
        }
    }
}

/// Tag component that lets the engine know that this is the player
#[derive(Component, Debug, Copy, Clone)]
pub struct Player;


/// Signal that an entity can receive.
/// Represents an instruction to carry out.
/// Either converted from user input, or emitted directly from an AI.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Signal {
    Move { direction: f32, speed: f32 },
    Jump { velocity: f32 }
}

#[derive(Component, Debug)]
pub struct Platformer {
    pub top_speed: f32
}

/// State a being is in
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum State {
    Idle,
    Running,
    Jumping,
    Attacking
}

impl Default for State {
    fn default() -> Self { Self::Idle }
}

/// 8-way direction
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Direction { E, NE, N, NW, W, SW, S, SE }
impl Direction {
    pub fn to_index(&self) -> usize {
        match self {
            Self::E => 0,
            Self::NE => 1,
            Self::N => 2,
            Self::NW => 3,
            Self::W => 4,
            Self::SW => 5,
            Self::S => 6,
            Self::SE => 7
        }
    }
    pub fn to_cardinal_direction(&self) -> Option<CardinalDirection> {
        match self {
            Self::E => Some(CardinalDirection::E),
            Self::N => Some(CardinalDirection::N),
            Self::W => Some(CardinalDirection::W),
            Self::S => Some(CardinalDirection::S),
            _ => None
        }
    }
}

/// 4-way direction
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum CardinalDirection { E, N, W, S }
impl CardinalDirection {
    pub fn to_index(&self) -> usize {
        match self {
            Self::E => 0,
            Self::N => 1,
            Self::W => 2,
            Self::S => 3
        }
    }
    pub fn to_direction(&self) -> Direction {
        match self {
            Self::E => Direction::E,
            Self::N => Direction::N,
            Self::W => Direction::W,
            Self::S => Direction::S
        }
    }
}

fn update_platformers(mut platformer_entities: Query<(&Platformer, &mut Velocity, &Friction)>) {
    for (platformer, mut velocity, friction) in platformer_entities.iter_mut() {
        let speed = platformer.top_speed / friction.xz - platformer.top_speed;
        velocity.0.x += speed;
    }
}

// ts = 10
// f = 0.9
// (ts + s) * f = ts
// ts + s = ts / f
// s = ts/f - ts