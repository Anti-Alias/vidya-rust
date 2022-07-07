use bevy::math::Vec3;

use super::{ Aabb, CylinderCollider, Collision };

pub fn collide_slope_with_cylinder(ter_bounds: Aabb, cyl: &CylinderCollider, delta: Vec3) -> Option<Collision> {
    None
}