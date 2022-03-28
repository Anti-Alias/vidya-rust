use std::fmt::Debug;

use bevy::prelude::*;

use crate::physics::TerrainPiece;

const T_EPSILON: f32 = 0.001;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3
}

impl Aabb {

    /// Center of the Aabb
    pub fn center(&self) -> Vec3 {
        self.min.lerp(self.max, 0.5)
    }

    /// Linear interpolation of an AABB
    pub fn lerp(&self, other: &Aabb, t: f32) -> Aabb {
        Aabb {
            min: self.min.lerp(other.min, t),
            max: self.max.lerp(other.max, t)
        }
    }
}

/// Collider of a vertical cylinder
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CylinderCollider {
    /// Center of the cylinder
    pub center: Vec3,
    pub radius: f32,
    pub half_height: f32,
}

impl CylinderCollider {

    pub fn aabb(&self) -> Aabb {
        let half_height = self.half_height;
        let min = Vec3::new(
            self.center.x - self.radius,
            self.center.y - half_height,
            self.center.z - self.radius
        );
        let max = Vec3::new(
            self.center.x + self.radius,
            self.center.y + half_height,
            self.center.z + self.radius
        );
        Aabb { min, max }
    }

    /// self + delta * t
    pub fn cast(&self, delta: Vec3, t: f32) -> Self {
        Self {
            center: self.center + delta * t,
            radius: self.radius,
            half_height: self.half_height
        }
    }

    /// self + delta
    pub fn mov(&self, delta: Vec3) -> Self {
        CylinderCollider {
            center: self.center + delta,
            radius: self.radius,
            half_height: self.half_height
        }
    }
}

/// Movement of a collider
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Movement<C: Debug + Copy + Clone + PartialEq> {
    pub origin: C,
    pub delta: Vec3
}

/// Collider of a [`TerrainPiece`].
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TerrainCollider {
    pub piece: TerrainPiece,
    pub position: Vec3,
    pub size: Vec3
}

impl TerrainCollider {

    /// Collides a terrain piece with a cylinder's movement
    pub fn collide_with_cylinder(&self, movement: &Movement<CylinderCollider>) -> Option<Collision> {
        match self.piece {
            TerrainPiece::Cuboid => self.collide_cuboid_with_cylinder(movement),
            TerrainPiece::Slope => self.collide_slope_with_cylinder(movement),
            _ => None
        }
    }

    fn collide_cuboid_with_cylinder(&self, movement: &Movement<CylinderCollider>) -> Option<Collision> {

        // Terrain bounds
        let ter_bounds = self.aabb();

        // Unpacks movement
        let delta = movement.delta;
        let prev_cyl = movement.origin;
        let cur_cyl_center = prev_cyl.center + delta;
        let cyl_hh = movement.origin.half_height;
        let cyl_rad = movement.origin.radius;

        // Collision code for left and right sides of this cuboid
        let x_collision = |ter_edge: f32| {
            let t = (ter_edge - prev_cyl.center.x) / delta.x;
            if t >= 0.0 && t < 1.0 {
                let lerped_center = prev_cyl.center + delta * t;
                let lerped_bottom = lerped_center.y - cyl_hh;
                let lerped_top = lerped_center.y + cyl_hh;
                let in_yz_bounds =
                    lerped_center.z > ter_bounds.min.z &&
                    lerped_center.z < ter_bounds.max.z &&
                    lerped_bottom < ter_bounds.max.y &&
                    lerped_top > ter_bounds.min.y;
                if in_yz_bounds {
                    let mut final_center = cur_cyl_center;
                    final_center.x = ter_edge;
                    let velocity =
                        if t > T_EPSILON { (final_center - lerped_center) / (1.0 - t) }
                        else { Vec3::new(0.0, delta.y, delta.z) };
                    return Some(Collision {
                        t,
                        velocity,
                    });
                }
            }
            None
        };

        // Collision code for left and right sides of this cuboid
        let z_collision = |ter_edge: f32| {
            let t = (ter_edge - prev_cyl.center.z) / delta.z;
            if t >= 0.0 && t < 1.0 {
                let lerped_center = prev_cyl.center + delta * t;
                let lerped_bottom = lerped_center.y - cyl_hh;
                let lerped_top = lerped_center.y + cyl_hh;
                let in_xy_bounds =
                    lerped_center.x > ter_bounds.min.x &&
                    lerped_center.x < ter_bounds.max.x &&
                    lerped_bottom < ter_bounds.max.y &&
                    lerped_top > ter_bounds.min.y;
                if in_xy_bounds {
                    let mut final_center = cur_cyl_center;
                    final_center.z = ter_edge;
                    let velocity =
                        if t > T_EPSILON { (final_center - lerped_center) / (1.0 - t) }
                        else { Vec3::new(delta.x, delta.y, 0.0) };
                    return Some(Collision {
                        t,
                        velocity
                    });
                }
            }
            None
        };

        // Left collision
        let coll = x_collision(ter_bounds.min.x - cyl_rad);
        if coll.is_some() {
            return coll;
        }

        // Right collision
        let coll = x_collision(ter_bounds.max.x + cyl_rad);
        if coll.is_some() {
            return coll;
        }

        // Near collision
        let coll = z_collision(ter_bounds.max.z + cyl_rad);
        if coll.is_some() {
            return coll;
        }

        // Far collision
        let coll = z_collision(ter_bounds.min.z - cyl_rad);
        if coll.is_some() {
            return coll;
        }

        None
    }

    fn collide_slope_with_cylinder(&self, movement: &Movement<CylinderCollider>) -> Option<Collision> {
        None
    }

    fn aabb(&self) -> Aabb {
        Aabb { min: self.position, max: self.position + self.size }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Collision {
    pub t: f32,
    pub velocity: Vec3
}

#[test]
fn collide_left() {
    let old_cylinder = CylinderCollider {
        center: Vec3::new(-15.0, 5.0, 5.0),
        half_height: 5.0,
        radius: 10.0
    };
    let mov = Movement {
        origin: old_cylinder,
        delta: Vec3::new(10.0, 0.0, 5.0)
    };
    let coll = TerrainCollider {
        piece: TerrainPiece::Cuboid,
        position: Vec3::new(0.0, 0.0, 0.0),
        size: Vec3::new(10.0, 10.0, 10.0)
    };
    let collision = coll.collide_with_cylinder(&mov);
    assert_eq!(
        Some(Collision {
            t: 0.5,
            velocity: Vec3::new(0.0, 0.0, 5.0)
        }),
        collision
    );
}

#[test]
fn collide_right() {
    let old_cylinder = CylinderCollider {
        center: Vec3::new(25.0, 5.0, 5.0),
        half_height: 5.0,
        radius: 10.0
    };
    let mov = Movement {
        origin: old_cylinder,
        delta: Vec3::new(-10.0, 0.0, 5.0)
    };
    let coll = TerrainCollider {
        piece: TerrainPiece::Cuboid,
        position: Vec3::new(0.0, 0.0, 0.0),
        size: Vec3::new(10.0, 10.0, 10.0)
    };
    let collision = coll.collide_with_cylinder(&mov);
    assert_eq!(
        Some(Collision {
            t: 0.5,
            velocity: Vec3::new(0.0, 0.0, 5.0)
        }),
        collision
    );
}

#[test]
fn collide_near() {
    let old_cylinder = CylinderCollider {
        center: Vec3::new(5.0, 5.0, 20.0),
        half_height: 5.0,
        radius: 10.0
    };
    let mov = Movement {
        origin: old_cylinder,
        delta: Vec3::new(5.0, 0.0, -15.0)
    };
    let coll = TerrainCollider {
        piece: TerrainPiece::Cuboid,
        position: Vec3::new(0.0, 0.0, 0.0),
        size: Vec3::new(10.0, 10.0, 10.0)
    };
    let collision = coll.collide_with_cylinder(&mov);
    assert_eq!(
        Some(Collision {
            t: 0.0,
            velocity: Vec3::new(5.0, 0.0, 0.0)
        }),
        collision
    );
}