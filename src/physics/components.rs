use bevy::prelude::*;

#[derive(Component, Debug, PartialEq, Clone, Copy, Default, Reflect)]
#[reflect(Component, PartialEq)]
pub struct Position(pub Vec3);

#[derive(Component, Debug, PartialEq, Clone, Copy, Default, Reflect)]
#[reflect(Component, PartialEq)]
pub struct PreviousPosition(pub Vec3);

#[derive(Component, Debug, PartialEq, Clone, Copy, Default, Reflect)]
#[reflect(Component, PartialEq)]
pub struct CylinderShape {
    pub half_height: f32,
    pub radius: f32
}

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
pub struct PhysicsBundle<Shape: Component> {
    pub position: Position,
    pub prev_position: PreviousPosition,
    pub shape: Shape,
    pub velocity: Velocity,
    pub friction: Friction,
    pub weight: Weight
}
impl<Shape: Component> PhysicsBundle<Shape> {
    pub fn new(
        position: Position,
        shape: Shape,
        friction: Friction,
        weight: Weight
    ) -> Self {
        Self {
            position,
            prev_position: PreviousPosition(position.0),
            shape,
            velocity: Velocity(Vec3::ZERO),
            friction,
            weight,
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

/// Determines side(s) that an entity is touching
#[derive(Component, PartialEq, Eq, Debug, Copy, Clone, Default)]
pub struct WallState {
    pub prev_on_ground: bool,
    pub on_ground: bool
}

impl WallState {
    /// Turns off flags necessary for jumping to be performed properly
    pub fn jump(&mut self) {
        self.on_ground = false;
        self.prev_on_ground = false;
    }
}


/// Global resource that determines how fast entities with a [`Weight`] will fall.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Gravity {
    pub gravity: f32
}
impl Default for Gravity {
    fn default() -> Self { Self{ gravity: 1.0 }}
}

/// Component that allows for components to "cast" their collider downwards a particular number of units.
/// This is useful for objects that want to follow the contour of the ground.
#[derive(Clone, Debug, Component, PartialEq)]
pub struct Caster {
    /// Distance to be casted downward.
    /// Should be positive.
    pub distance: f32
}

impl Default for Caster {
    fn default() -> Self {
        Self { distance: 4.0 }
    }
}


// ----------------- Systems -----------------


pub fn apply_gravity(
    gravity: Res<Gravity>,
    mut entities: Query<(&Weight, &mut Velocity)>
) {
    log::debug!("(SYSTEM) apply_gravity");
    for (weight, mut velocity) in entities.iter_mut() {
        let vel = &mut velocity.0;
        vel.y -= gravity.gravity * weight.0;
    }
}


/// Applies friction to entities
pub fn apply_friction(mut query: Query<(&mut Velocity, &Friction), With<Position>>) {
    log::debug!("(SYSTEM) apply_friction");
    for (mut velocity, friction) in query.iter_mut() {
        let vel = &mut velocity.0;
        vel.x *= friction.xz;
        vel.z *= friction.xz;
        vel.y *= friction.y;
    }
}

/// Sets previous position data to current position data.
pub fn prepare_positions(mut query: Query<(&mut Position, &mut PreviousPosition)>) {
    log::debug!("(SYSTEM) prepare_positions");
    for (position, mut prev_position) in query.iter_mut() {
        prev_position.0 = position.0;
    }
}

/// Resets physics states.
pub fn prepare_states(mut query: Query<&mut WallState>) {
    log::debug!("(SYSTEM) prepare_states");
    for mut state in query.iter_mut() {
        state.prev_on_ground = state.on_ground;
        state.on_ground = false;
    }
}

/// Moves an entity based on it's velocity
pub fn apply_velocity(
    mut query: Query<(&mut Position, &Velocity)>
) {
    log::debug!("(SYSTEM) apply_velocity");
    for (mut position, velocity) in query.iter_mut() {
        position.0 += velocity.0;
    }
}