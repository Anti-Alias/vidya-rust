mod cuboid;
mod slope;

use std::fmt::Debug;

use bevy::{prelude::*, math::Vec3Swizzles, utils::HashSet};

use crate::physics::{ Terrain, Coords, TerrainPiece, TerrainPieceRef };
use cuboid::collide_cuboid_with_cylinder;
use slope::collide_slope_with_cylinder;

const T_EPSILON: f32 = 0.0001;

/// Represents a collision event
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Collision {
    /// T value between 0 and 1 which determines when the collision occurred
    pub t: f32,
    /// Resulting velocity after the collision
    pub velocity: Vec3,
    /// Positional offset to apply after collision
    pub offset: Vec3,
    /// Type of surface that was hit at collision
    pub typ: CollisionType
}

impl Collision {
    fn new(t: f32, velocity: Vec3, typ: CollisionType) -> Self {
        Self { t, velocity, offset: Vec3::ZERO, typ }
    }
}


/// Type of collision that occurred
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum CollisionType { Floor, Wall, Ceiling }

// 3D axis aligned bounding box
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

/// Represents a a slope collider
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SlopeCollider {
    /// Bottom-left-near corner
    pub edge: Vec3,

    /// Size of the collider
    pub size: Vec3
}

/// Collider of a [`TerrainPiece`].
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PieceCollider {
    pub piece: TerrainPiece,
    pub position: Vec3,
    pub size: Vec3
}

impl PieceCollider {

    /// Collides a terrain piece with a cylinder's movement
    pub fn collide_with_cylinder(&self, cyl: &CylinderCollider, delta: Vec3) -> Option<Collision> {
        match self.piece {
            TerrainPiece::Cuboid => collide_cuboid_with_cylinder(self.aabb(), cyl, delta),
            TerrainPiece::Slope => collide_slope_with_cylinder(self.aabb(), cyl, delta),
            _ => None
        }
    }


    fn aabb(&self) -> Aabb {
        Aabb { min: self.position, max: self.position + self.size }
    }
}

/// Uniquely defines the terrain that was collided with.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct TerrainId(Coords);

pub trait TerrainCollider {
    fn collide_with_cylinder(
        &self,
        cylinder: &CylinderCollider,
        delta: Vec3,
        exclude: &HashSet<TerrainId>
    ) -> Option<(Collision, TerrainId)>;
}

impl TerrainCollider for Terrain {

    fn collide_with_cylinder(
        &self,
        cylinder: &CylinderCollider,
        delta: Vec3,
        exclude: &HashSet<TerrainId>
    ) -> Option<(Collision, TerrainId)> {
        
        let mut result: Option<(Collision, TerrainId)> = None;
        let mut terRef = None;

        // Determines terrain area to select based on cylinder's size and movement
        let piece_size = self.piece_size();
        let cyl_aabb = moving_cylinder_aabb(cylinder, delta);
        let (min, max) = Coords::from_aabb(cyl_aabb, piece_size);

        // For all terrain pieces in the selection...
        for piece_ref in self.iter_pieces(min, max) {
            let tid = TerrainId(piece_ref.coords);
            if exclude.contains(&tid) { continue };

            // Create short-lived piece collider
            let TerrainPieceRef { piece, coords } = piece_ref;
            let piece_pos = Vec3::new(
                coords.x as f32 * piece_size.x,
                coords.y as f32 * piece_size.y,
                coords.z as f32 * piece_size.z
            );
            let piece_coll = PieceCollider {
                piece: *piece,
                position: piece_pos,
                size: piece_size
            };

            // Collides the piece collider with the moving cylinder
            let collision = match piece_coll.collide_with_cylinder(cylinder, delta) {
                Some(coll) => coll,
                None => continue
            };
            if closer_than(collision, result.map(|data| data.0)) {
                result = Some((collision, tid));
                terRef = Some(piece_ref);
            }
        }
        
        // Returns closest collision
        println!("Terrain ref: {:?}", terRef);
        result
    }
}

// Computes the Aabb of a moving cylinder
fn moving_cylinder_aabb(cylinder: &CylinderCollider, delta: Vec3) -> Aabb {
    let min1 = Vec3::new(
        cylinder.center.x - cylinder.radius,
        cylinder.center.y - cylinder.half_height,
        cylinder.center.z - cylinder.radius
    );
    let max1 = Vec3::new(
        cylinder.center.x + cylinder.radius,
        cylinder.center.y + cylinder.half_height,
        cylinder.center.z + cylinder.radius
    );
    let min2 = min1 + delta;
    let max2 = max1 + delta;
    let min = min1.min(min2);
    let max = max1.max(max2);
    Aabb { min, max }
}

fn closer_than(a: Collision, b: Option<Collision>) -> bool {
    match b {
        Some(b) => {
            if a.t < b.t { true }
            else { false }
        }
        None => true
    }
}

// Compares two floats with a margin of error
fn float_eq(a: f32, b: f32, epsilon: f32) -> bool {
    (a - b).abs() < epsilon
}

fn t_in_range(t: f32) -> bool {
    const EPSILON: f32 = 0.001;
    t >= 0.0 - EPSILON && t <= 1.0 + EPSILON
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Collision2D {
    pub t: f32,
    pub velocity: Vec2
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct CircleHelper {
    center: Vec2,
    radius: f32
}

impl CircleHelper {
    fn contains_point(&self, point: Vec2) -> bool {
        let dist_squared = (point - self.center).length_squared();
        let rad_squared = self.radius*self.radius;
        dist_squared <= rad_squared
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct RectHelper {
    pub min: Vec2,
    pub max: Vec2
}

impl RectHelper {
    fn contains_point(&self, point: Vec2) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y
    }
}

fn collide_line_with_circle(src: Vec2, dest: Vec2, circle: CircleHelper) -> Option<Collision2D> {

    // Gets closest point on line "d"
    const EPSILON: f32 = 0.001;
    let a = src;
    let c = circle.center;
    let ca = c - a;
    let rad_squared = circle.radius * circle.radius;
    if ca.length_squared() + EPSILON < rad_squared {
        return None;
    };
    let b = dest;
    let ba = b - a;
    let ba_len = ba.length();
    let ba_norm = ba / ba_len;
    let d = {
        let dist = ca.dot(ba_norm);
        src + ba_norm * dist
    };
    let dc_len_sq = (d - c).length_squared();
    if dc_len_sq >= rad_squared {
        return None;
    }

    // Gets lengths of right triangle to compute "back distance"
    let ec_len_sq = rad_squared;
    let ed_len = (ec_len_sq - dc_len_sq).sqrt();    // Back distance

    // Computes collision point
    let e = d - ba_norm * ed_len;
    let ea_len = (e - a).length();
    let t = ea_len / ba_len;
    if !t_in_range(t) {
        return None;
    }

    // Calculates velocity
    let ce_norm = (c - e).normalize();
    let ce_norm_3d = Vec3::new(ce_norm.x, ce_norm.y, 0.0);
    let ba_norm_3d = Vec3::new(ba_norm.x, ba_norm.y, 0.0);
    let up = ce_norm_3d.cross(ba_norm_3d);
    let velocity = if up == Vec3::ZERO {
        Vec2::ZERO
    }
    else {
        let vel_norm = up.cross(ce_norm_3d).xy().normalize();
        let vel_mult = 1.0 - ba_norm.dot(ce_norm);
        vel_norm * ba_len * vel_mult
    };

    // Done
    Some(Collision2D {
        t,
        velocity
    })
}