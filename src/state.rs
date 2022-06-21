use bevy::prelude::*;

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

/// Component that holds state
#[derive(Component, Debug, Clone, Default)]
pub struct ActionState(pub State);