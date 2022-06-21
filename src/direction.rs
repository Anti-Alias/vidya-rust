pub use bevy::prelude::*;

/// Component that allows an Entity to face a direction and hold state
/// IE: Player, Creatures, etc
#[derive(Component, Default)]
pub struct DirectionState {
    /// Direction the entity is facing in radians.
    pub direction: f32
}

impl DirectionState {

    /// Determines group animation index to use for the direction of this being.
    pub fn get_direction_index(&self, direction_type: DirectionType) -> usize {
        match direction_type {
            DirectionType::EightWay => Direction::from_radians(self.direction).to_index(),
            DirectionType::FourWay => CardinalDirection::from_radians(self.direction).to_index()
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

    pub fn from_radians(radians: f32) -> Self {
        let pi = std::f32::consts::PI;
        let pi2 = pi*2.0;
        let slice = pi2/8.0;
        let halfslice = slice / 2.0;
        let direction = ((radians % pi2) + pi2) % pi2 + halfslice;
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

    pub fn to_radians(&self) -> f32 {
        use std::f32::consts::PI;
        match self {
            Self::E => 0.0*PI,
            Self::N => 0.5*PI,
            Self::W => 1.0*PI,
            Self::S => 1.5*PI,
        }
    }

    pub fn from_radians(radians: f32) -> Self {
        const EPSILON: f32 = 0.00001;
        let pi = std::f32::consts::PI;
        let pi2 = pi*2.0;
        let slice = pi2/4.0;
        let halfslice = slice / 2.0;
        let direction = ((radians % pi2) + pi2) % pi2;
        if direction < slice*1.0-halfslice || direction > slice*4.0-halfslice-EPSILON {
            CardinalDirection::E
        }
        else if direction >= slice*1.0-halfslice && direction <= slice*2.0-halfslice-EPSILON {
            CardinalDirection::N
        }
        else if direction > slice*2.0-halfslice-EPSILON && direction < slice*3.0-halfslice+EPSILON {
            CardinalDirection::W
        }
        else {
            CardinalDirection::S
        }
    }

    pub fn from_keyboard(input: &Input<KeyCode>) -> Option<Self> {
        
        let mut x = 0;
        let mut y = 0;
        if input.pressed(KeyCode::Left) {
            x -= 1;
        }
        if input.pressed(KeyCode::Right) {
            x += 1;
        }
        if input.pressed(KeyCode::Down) {
            y -= 1;
        }
        if input.pressed(KeyCode::Up) {
            y += 1;
        }

        match (x, y) {
            (-1, 0) => Some(Self::W),
            (1, 0) => Some(Self::E),
            (0, -1) => Some(Self::S),
            (0, 1) => Some(Self::N),

            (1, 1) => {
                if input.just_pressed(KeyCode::Right) {
                    Some(Self::E)
                }
                else if input.just_pressed(KeyCode::Up) {
                    Some(Self::N)
                }
                else { 
                    None
                }
            },

            (-1, 1) => {
                if input.just_pressed(KeyCode::Left) {
                    Some(Self::W)
                }
                else if input.just_pressed(KeyCode::Up) {
                    Some(Self::N)
                }
                else { 
                    None
                }
            },

            (-1, -1) => {
                if input.just_pressed(KeyCode::Left) {
                    Some(Self::W)
                }
                else if input.just_pressed(KeyCode::Down) {
                    Some(Self::S)
                }
                else { 
                    None
                }
            },

            (1, -1) => {
                if input.just_pressed(KeyCode::Right) {
                    Some(Self::E)
                }
                else if input.just_pressed(KeyCode::Down) {
                    Some(Self::S)
                }
                else { 
                    None
                }
            },

            _ => None
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

/// Type of direction
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum DirectionType {
    EightWay,
    FourWay
}