use bevy::math::{ Vec2, Vec3, Vec3Swizzles };

use super::{ Aabb, CylinderCollider, Collision, CollisionType, RectHelper, CircleHelper, collide_line_with_circle, T_EPSILON };

pub fn collide_slope_with_cylinder(ter_bounds: Aabb, cyl: &CylinderCollider, delta: Vec3) -> Option<Collision> {
    
    // Unpacks movement
    let next_cyl_center = cyl.center + delta;

    // Collision code for left and right sides of this slope
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

    // Collision code for the top side of this slope
    let top_collision = |ter_edge: f32, coll_type: CollisionType| {
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

    // Collision code for the bottom side of this slope
    let bottom_collision = |ter_edge: f32, coll_type: CollisionType| {
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

    // Collision code for near and far sides of this slope
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
        let coll = bottom_collision(ter_bounds.min.y - cyl.half_height, CollisionType::Ceiling);
        if coll.is_some() {
            return coll;
        }
    }

    // Top collision
    if delta.y < 0.0 {
        let coll = top_collision(ter_bounds.max.y + cyl.half_height, CollisionType::Floor);
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

fn slope_intercept_of(a: Vec2, b: Vec2) -> (f32, f32) {
    let diff = b - a;
    let slope = diff.y / diff.x;
    let intercept = a.y - slope*a.x;
    (slope, intercept)
}

// Checks if c is between c and a
fn between(mut a: f32, mut b: f32, c: f32) -> bool {
    if a > b {
        std::mem::swap(&mut a, &mut b);
    }
    c >= a && c <= b
}

fn intersect(a1: Vec2, b1: Vec2, a2: Vec2, b2: Vec2) -> Option<f32> {

    // Checks pathological cases
    if a1.x == b1.x {
        if a2.x == b2.x {
            return None;
        }
        let t = (a1.x - a2.x) / (b2.x - a2.x);
        if t >= 0.0 && t <= 1.0 {
            return Some(t);
        }
        return None;
    }
    else if a2.x == b2.x {
        if a1.x == b1.x {
            return None;
        }
        let t = (a2.x - a1.x) / (b1.x - a1.x);
        if t >= 0.0 && t <= 1.0 {
            return Some(t);
        }
        return None;
    }

    // Normal slope intersection
    let (slope1, inter1) = slope_intercept_of(a1, b1);
    let (slope2, inter2) = slope_intercept_of(a2, b2);
    let x = (inter2 - inter1) / (slope1 - slope2);
    if between(a1.x, b1.x, x) && between(a2.x, b2.x, x) {
        Some((x - a1.x) / (b1.x - a1.x))
    }
    else {
        None
    }
}

#[test]
fn test_intersect_1() {
    let a1 = Vec2::new(2.0, 1.0);
    let b1 = Vec2::new(3.0, 4.0);
    let a2 = Vec2::new(1.0, 3.0);
    let b2 = Vec2::new(3.0, 2.0);
    let intersection = intersect(a1, b1, a2, b2);
    assert_eq!(
        Some(0.42857146),
        intersection
    );
}

#[test]
fn test_intersect_2() {
    let a1 = Vec2::new(0.0, 0.0);
    let b1 = Vec2::new(4.0, 4.0);
    let a2 = Vec2::new(4.0, 0.0);
    let b2 = Vec2::new(0.0, 4.0);
    let intersection = intersect(a1, b1, a2, b2);
    assert_eq!(
        Some(0.5),
        intersection
    );
}

#[test]
fn test_intersect_3() {
    let a1 = Vec2::new(1.0, 1.0);
    let b1 = Vec2::new(7.0, 7.0);
    let a2 = Vec2::new(4.0, 0.0);
    let b2 = Vec2::new(3.0, 1.0);
    let intersection = intersect(a1, b1, a2, b2);
    assert_eq!(
        None,
        intersection
    );
}

#[test]
fn test_intersect_4() {
    let a1 = Vec2::new(1.0, 1.0);
    let b1 = Vec2::new(0.0, 0.0);
    let a2 = Vec2::new(2.0, 0.0);
    let b2 = Vec2::new(1.0, 1.0);
    let intersection = intersect(a1, b1, a2, b2);
    assert_eq!(
        Some(0.0),
        intersection
    );
}

#[test]
fn test_intersect_5() {
    let a1 = Vec2::new(0.999, 1.0);
    let b1 = Vec2::new(0.0, 0.0);
    let a2 = Vec2::new(2.0, 0.0);
    let b2 = Vec2::new(1.0, 1.0);
    let intersection = intersect(a1, b1, a2, b2);
    assert_eq!(
        None,
        intersection
    );
}