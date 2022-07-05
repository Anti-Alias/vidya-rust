use bevy::math::{Vec2, Vec3, Vec3Swizzles};

use super::{Aabb, CylinderCollider, Collision, CollisionType, RectHelper, CircleHelper, collide_line_with_circle};

const T_EPSILON: f32 = 0.001;


pub fn collide_cuboid_with_cylinder(ter_bounds: Aabb, cyl: &CylinderCollider, delta: Vec3) -> Option<Collision> {

    // Unpacks movement
    let next_cyl_center = cyl.center + delta;

    // Collision code for left and right sides of this cuboid
    let x_collision = |ter_edge: f32| {
        let t = (ter_edge - cyl.center.x) / delta.x;
        if t >= 0.0 && t <= 1.0 {
            let t = t - T_EPSILON;
            let lerped_center = cyl.center + delta * t;
            let lerped_bottom = lerped_center.y - cyl.half_height;
            let lerped_top = lerped_center.y + cyl.half_height;
            let in_yz_bounds =
                lerped_center.z > ter_bounds.min.z &&
                lerped_center.z < ter_bounds.max.z  &&
                lerped_bottom < ter_bounds.max.y &&
                lerped_top > ter_bounds.min.y;
            if in_yz_bounds {
                return Some(Collision {
                    t,
                    velocity: Vec3::new(0.0, delta.y, delta.z),
                    typ: CollisionType::Wall
                });
            }
        }
        None
    };

    // Collision code for bottom and top sides of this cuboid
    let y_collision = |ter_edge: f32, coll_type: CollisionType| {
        let t = (ter_edge - cyl.center.y) / delta.y;
        if t >= 0.0 && t <= 1.0 {
            let lerped_center = cyl.center + delta * t;
            let lerped_center_xz = lerped_center.xz();
            let in_xz_bounds =
                RectHelper {
                    min: Vec2::new(ter_bounds.min.x - cyl.radius, ter_bounds.min.z),
                    max: Vec2::new(ter_bounds.max.x + cyl.radius, ter_bounds.max.z)
                }.contains_point(lerped_center_xz) ||
                RectHelper {
                    min: Vec2::new(ter_bounds.min.x, ter_bounds.min.z - cyl.radius),
                    max: Vec2::new(ter_bounds.max.x, ter_bounds.max.z + cyl.radius)
                }.contains_point(lerped_center_xz) ||
                CircleHelper {
                    center: ter_bounds.min.xz(),
                    radius: cyl.radius
                }.contains_point(lerped_center_xz) ||
                CircleHelper {
                    center: Vec2::new(ter_bounds.max.x, ter_bounds.min.z),
                    radius: cyl.radius
                }.contains_point(lerped_center_xz) ||
                CircleHelper {
                    center: Vec2::new(ter_bounds.min.x, ter_bounds.max.z),
                    radius: cyl.radius
                }.contains_point(lerped_center_xz) ||
                CircleHelper {
                    center: ter_bounds.max.xz(),
                    radius: cyl.radius
                }.contains_point(lerped_center_xz);
            if in_xz_bounds {
                return Some(Collision {
                    t,
                    velocity: Vec3::new(delta.x, 0.0, delta.z),
                    typ: coll_type
                });
            }
        }
        None
    };

    // Collision code for near and far sides of this cuboid
    let z_collision = |ter_edge: f32| {
        let t = (ter_edge - cyl.center.z) / delta.z;
        if t >= 0.0 && t <= 1.0 {
            let lerped_center = cyl.center + delta * t;
            let lerped_bottom = lerped_center.y - cyl.half_height;
            let lerped_top = lerped_center.y + cyl.half_height;
            let in_xy_bounds =
                lerped_center.x > ter_bounds.min.x &&
                lerped_center.x < ter_bounds.max.x &&
                lerped_bottom < ter_bounds.max.y &&
                lerped_top > ter_bounds.min.y;
            if in_xy_bounds {
                return Some(Collision {
                    t,
                    velocity: Vec3::new(delta.x, delta.y, 0.0),
                    typ: CollisionType::Wall
                });
            }
        }
        None
    };

    // Collision of a point with a circle
    let edge_collision = |edge: Vec2| -> Option<Collision> {
        let cir = CircleHelper {
            center: edge,
            radius: cyl.radius
        };
        let coll_2d = collide_line_with_circle(cyl.center.xz(), next_cyl_center.xz(), cir)?;
        let lerped_center = cyl.center + delta * coll_2d.t;
        let lerped_bottom = lerped_center.y - cyl.half_height;
        let lerped_top = lerped_center.y + cyl.half_height;
        let in_y_bounds =
            lerped_bottom < ter_bounds.max.y &&
            lerped_top > ter_bounds.min.y;
        if in_y_bounds {
            return Some(Collision {
                t: coll_2d.t,
                velocity: Vec3::new(coll_2d.velocity.x, delta.y, coll_2d.velocity.y),
                typ: CollisionType::Wall
            })
        }
        None
    };

    // Left collision
    if delta.x > 0.0 {
        let coll = x_collision(ter_bounds.min.x - cyl.radius);
        if coll.is_some() {
            return coll;
        }
    }

    // Right collision
    if delta.x < 0.0 {
        let coll = x_collision(ter_bounds.max.x + cyl.radius);
        if coll.is_some() {
            return coll;
        }
    }

    // Bottom collision
    if delta.y > 0.0 {
        let coll = y_collision(ter_bounds.min.y - cyl.half_height, CollisionType::Ceiling);
        if coll.is_some() {
            return coll;
        }
    }
    

    // Top collision
    if delta.y < 0.0 {
        let coll = y_collision(ter_bounds.max.y + cyl.half_height, CollisionType::Floor);
        if coll.is_some() {
            return coll;
        }
    }

    // Near collision
    if delta.z < 0.0 {
        let coll = z_collision(ter_bounds.max.z + cyl.radius);
        if coll.is_some() {
            return coll;
        }
    }

    // Far collision
    if delta.z > 0.0 {
        let coll = z_collision(ter_bounds.min.z - cyl.radius);
        if coll.is_some() {
            return coll;
        }
    }

    // Far/left corner collision
    let coll = edge_collision(Vec2::new(ter_bounds.min.x, ter_bounds.min.z));
    if coll.is_some() {
        return coll;
    }

    // Far/right corner collision
    let coll = edge_collision(Vec2::new(ter_bounds.max.x, ter_bounds.min.z));
    if coll.is_some() {
        return coll;
    }

    // Near/left corner collision
    let coll = edge_collision(Vec2::new(ter_bounds.min.x, ter_bounds.max.z));
    if coll.is_some() {
        return coll;
    }

    // Near/right corner collision
    let coll = edge_collision(Vec2::new(ter_bounds.max.x, ter_bounds.max.z));
    if coll.is_some() {
        return coll;
    }

    // Default
    None
}