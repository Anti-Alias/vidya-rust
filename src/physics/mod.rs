use bevy::prelude::*;
use crate::app::{ AppState, AppLabel };


pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(SystemSet::on_update(AppState::AppRunning)
                .with_system(apply_friction.label(AppLabel::PhysicsFriction))
                .with_system(apply_velocity.label(AppLabel::PhysicsVelocity).after(AppLabel::PhysicsFriction))
                .with_system(sync_transform.label(AppLabel::PhysicsSync).after(AppLabel::PhysicsVelocity))
            )
        ;
    }
}

/// Represents an axis-aligned bounding box
#[derive(Component, PartialEq, Debug, Copy, Clone)]
pub struct Position(Vec3);

/// Velocity of an entity
#[derive(Component, PartialEq, Debug, Copy, Clone)]
pub struct Velocity(Vec3);

/// Friction of an entity
#[derive(Component, PartialEq, Debug, Copy, Clone)]
pub struct Friction(f32);

/// Computed boundary of an [`AABB`].
pub struct Bounds {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32
}

impl Bounds {
    pub fn center(&self) -> Vec3 {
        Vec3::new(
            (self.left + self.right) * 0.5,
            (self.bottom + self.top) * 0.5,
            (self.near + self.far) * 0.5,
        )
    }
}


// ----------------- Systems -----------------


// Applies friction to entities
pub fn apply_friction(mut query: Query<(&mut Velocity, &Friction), With<Position>>) {
    for (mut velocity, friction) in query.iter_mut() {
        velocity.0 *= friction.0;
    }
}

// Moves an entity based on it's velocity
pub fn apply_velocity(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut position, velocity) in query.iter_mut() {
        position.0 += velocity.0;
    }
}

// Synchronizes an Transformwith an [`AABB`]'s center.
pub fn sync_transform(mut query: Query<(&Position, &mut Transform)>) {
    for (position, mut transform) in query.iter_mut() {
        *transform = transform.with_translation(position.0);
    }
}