pub use bevy::prelude::*;

/// Component that allows an Entity to face a direction and hold state
/// IE: Player, Creatures, etc
#[derive(Component, Default)]
pub struct Being {
    /// Direction being is facing in radians, rotating along the Y axis in a counter-clockwise motion
    pub direction: f32,
    /// Current high-level action that entity is doing.
    /// Used to control what behaviors an Entity can and can't be doing at any given moment
    pub state: State,
}

impl Being {

    /// Rounded 8-way direction the [`Being`] is facing
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

    /// Rounded 4-way direction the [`Being`] is facing
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

/// 8-way direction
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Direction { E, NE, N, NW, W, SW, S, SE }
impl Direction {

    pub fn from_keyboard(input: &Input<KeyCode>) -> Option<Self> {
        let mut x = 0;
        let mut y = 0;
        if input.pressed(KeyCode::Up) { y += 1; }
        if input.pressed(KeyCode::Down) { y -= 1; }
        if input.pressed(KeyCode::Left) { x -= 1; }
        if input.pressed(KeyCode::Right) { x += 1; }
        match (x, y) {
            (1, 0) => Some(Self::E),
            (1, 1) => Some(Self::NE),
            (0, 1) => Some(Self::N),
            (-1, 1) => Some(Self::NW),
            (-1, 0) => Some(Self::W),
            (-1, -1) => Some(Self::SW),
            (0, -1) => Some(Self::S),
            (1, -1) => Some(Self::SE),
            _ => None
        }
    }

    pub fn to_radians(&self) -> f32 {
        use std::f32::consts::PI;
        match self {
            Self::E => 0.0*PI,
            Self::NE => 0.25*PI,
            Self::N => 0.5*PI,
            Self::NW => 0.75*PI,
            Self::W => 1.0*PI,
            Self::SW => 1.25*PI,
            Self::S => 1.5*PI,
            Self::SE => 1.75*PI
        }
    }

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