use bevy::prelude::*;
use crate::app::{ AppState, AppLabel };


pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Gravity::default())
            .add_system_set(SystemSet::on_update(AppState::AppRunning)
                .with_system(apply_friction.label(AppLabel::PhysicsFriction))
                .with_system(apply_gravity.label(AppLabel::PhysicsGravity).after(AppLabel::PhysicsFriction))
                .with_system(apply_velocity.label(AppLabel::PhysicsVelocity).after(AppLabel::PhysicsFriction))
                .with_system(sync_transform.label(AppLabel::PhysicsSync).after(AppLabel::PhysicsVelocity))
            )
        ;
    }
}

#[derive(Component, PartialEq, Debug, Copy, Clone, Default)]
pub struct Position(pub Vec3);

/// Velocity of an entity
#[derive(Component, PartialEq, Debug, Copy, Clone, Default)]
pub struct Velocity(pub Vec3);

/// Friction of an entity
#[derive(Component, PartialEq, Debug, Copy, Clone)]
pub struct Friction {
    pub xz: f32,
    pub y: f32
}

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
impl Default for Gravity {
    fn default() -> Self { Self{ gravity: 1.0 }}
}


// ----------------- Systems -----------------


// Applies friction to entities
pub fn apply_friction(mut query: Query<(&mut Velocity, &Friction), With<Position>>) {
    for (mut velocity, friction) in query.iter_mut() {
        let vel = &mut velocity.0;
        vel.x *= friction.xz;
        vel.z *= friction.xz;
        vel.y *= friction.y;
    }
}

// Moves an entity based on it's velocity
pub fn apply_velocity(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut position, velocity) in query.iter_mut() {
        position.0 += velocity.0;
    }
}

// Synchronizes a [`Transform`] with a [`Position`].
pub fn sync_transform(mut query: Query<(&Position, &mut Transform)>) {
for (position, mut transform) in query.iter_mut() {
        let position = Vec3::new(
            position.0.x.round(),
            position.0.y.round(),
            position.0.z.round()
        );
        *transform = transform.with_translation(position);
    }
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