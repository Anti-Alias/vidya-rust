use bevy::prelude::*;
use crate::app::{ AppState, AppLabel, TickTimer };


pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Gravity::default())
            .add_system_set(SystemSet::on_update(AppState::AppRunning)
                .with_system(apply_gravity.label(AppLabel::Logic).after(AppLabel::Input).after(AppLabel::Input))
                .with_system(apply_friction.label(AppLabel::PhysicsFriction).after(AppLabel::Logic))
                .with_system(apply_velocity.label(AppLabel::PhysicsVelocity).after(AppLabel::PhysicsFriction))
            )
        ;
    }
}

#[derive(Component, Debug, PartialEq, Clone, Copy, Default, Reflect)]
#[reflect(Component, PartialEq)]
pub struct Position(pub Vec3);

#[derive(Component, Debug, PartialEq, Clone, Copy, Default, Reflect)]
#[reflect(Component, PartialEq)]
pub struct PreviousPosition(pub Vec3);

/// Velocity of an entity
#[derive(Component, PartialEq, Debug, Copy, Clone, Default)]
pub struct Velocity(pub Vec3);

/// Determines how quickly an entity will fall
#[derive(Component, Debug, Copy, Clone, PartialEq)]
pub struct Weight(pub f32);
impl Default for Weight {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Bundle)]
pub struct PhysicsBundle {
    pub position: Position,
    pub prev_position: PreviousPosition,
    pub velocity: Velocity,
    pub friction: Friction,
    pub weight: Weight
}
impl PhysicsBundle {
    pub fn new(position: Position, velocity: Velocity, friction: Friction, weight: Weight) -> Self {
        Self {
            position,
            prev_position: PreviousPosition(position.0),
            velocity,
            friction,
            weight
        }
    }
}

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


/// Global resource that determines how fast entities with a [`Weight`] will fall.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Gravity {
    pub gravity: f32
}
impl Default for Gravity {
    fn default() -> Self { Self{ gravity: 1.0 }}
}


// ----------------- Systems -----------------


pub fn apply_gravity(
    tick_timer: Res<TickTimer>,
    gravity: Res<Gravity>,
    mut entities: Query<(&Weight, &mut Velocity)>
) {
    for _ in 0..tick_timer.times_finished() {
        for (weight, mut velocity) in entities.iter_mut() {
            let vel = &mut velocity.0;
            vel.y -= gravity.gravity * weight.0;
        }
    }
}


// Applies friction to entities
pub fn apply_friction(
    tick_timer: Res<TickTimer>,
    mut query: Query<(&mut Velocity, &Friction), With<Position>>
) {
    for _ in 0..tick_timer.times_finished() {
        for (mut velocity, friction) in query.iter_mut() {
            let vel = &mut velocity.0;
            vel.x *= friction.xz;
            vel.z *= friction.xz;
            vel.y *= friction.y;
        }
    }
}

// Moves an entity based on it's velocity
pub fn apply_velocity(
    tick_timer: Res<TickTimer>,
    mut query: Query<(&mut Position, &mut PreviousPosition, &Velocity)>
) {
    for _ in 0..tick_timer.times_finished() {
        for (mut position, mut prev_position, velocity) in query.iter_mut() {
            prev_position.0 = position.0;
            position.0 += velocity.0;
        }
    }
}