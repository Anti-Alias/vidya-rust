use bevy::prelude::*;

use crate::map::Velocity;

/// Plugin for player behavior
pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {

    }
}



/// Determines how quickly an entity will fall
#[derive(Component, Debug, Copy, Clone, PartialEq)]
pub struct Weight {
    pub weight: f32
}
impl Default for Weight {
    fn default() -> Self {
        Self {
            weight: 1.0
        }
    }
}


/// Global resource that determines how fast entities with a [`Weight`] will fall.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Gravity {
    pub gravity: f32
}


pub fn apply_gravity(
    gravity: Res<Gravity>,
    mut entities: Query<(&Weight, &mut Velocity)>
) {
    for (weight, mut velocity) in entities.iter_mut() {
        let vel = &mut velocity.0;
        vel.y -= gravity.gravity * weight.weight;
    }
}