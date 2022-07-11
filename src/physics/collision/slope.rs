use bevy::math::{ Vec2, Vec3, Vec3Swizzles };

use crate::physics::collision::float_eq;

use super::{ Aabb, CylinderCollider, Collision, CollisionType, RectHelper, CircleHelper, collide_line_with_circle, T_EPSILON, Collision2D };

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
                return Some(Collision::new(
                    t,
                    Vec3::new(0.0, delta.y, delta.z),
                    CollisionType::Wall
                ));
            }
        }
        None
    };

    // Helper callback function
    let in_xz_bounds = |point: Vec2| -> bool {
        RectHelper {
            min: Vec2::new(ter_bounds.min.x - cyl.radius, ter_bounds.min.z),
            max: Vec2::new(ter_bounds.max.x + cyl.radius, ter_bounds.max.z)
        }.contains_point(point) ||
        RectHelper {
            min: Vec2::new(ter_bounds.min.x, ter_bounds.min.z - cyl.radius),
            max: Vec2::new(ter_bounds.max.x, ter_bounds.max.z + cyl.radius)
        }.contains_point(point) ||
        CircleHelper {
            center: ter_bounds.min.xz(),
            radius: cyl.radius
        }.contains_point(point) ||
        CircleHelper {
            center: Vec2::new(ter_bounds.max.x, ter_bounds.min.z),
            radius: cyl.radius
        }.contains_point(point) ||
        CircleHelper {
            center: Vec2::new(ter_bounds.min.x, ter_bounds.max.z),
            radius: cyl.radius
        }.contains_point(point) ||
        CircleHelper {
            center: ter_bounds.max.xz(),
            radius: cyl.radius
        }.contains_point(point)
    };

    // Collision code for the top side of this slope
    let top_collision = || -> Option<Collision> {
        let min = ter_bounds.min;
        let max = ter_bounds.max;
        let a1 = Vec2::new(
            cyl.center.z - cyl.radius,
            cyl.center.y - cyl.half_height
        );
        let b1 = a1 + delta.zy();
        let a2 = Vec2::new(max.z, min.y);
        let b2 = Vec2::new(min.z, max.y);
        let coll2d = intersect(a1, b1, a2, b2)?;
        let vel_zy = coll2d.velocity;
        let ter_point = cyl.center + delta * coll2d.t;
            if in_xz_bounds(ter_point.xz()) {
                return Some(Collision {
                    t: coll2d.t,
                    velocity: Vec3::new(delta.x, vel_zy.y, vel_zy.x),
                    offset: Vec3::new(0.0, 0.01, 0.0),
                    typ: CollisionType::Floor
                });
            }
        None
    };
    

    // Collision code for the bottom side of this slope
    let bottom_collision = |ter_edge: f32, coll_type: CollisionType| {
        let t = (ter_edge - cyl.center.y) / delta.y;
        if t >= 0.0 && t <= 1.0 {
            let lerped_center = cyl.center + delta * t;
            if in_xz_bounds(lerped_center.xz()) {
                return Some(Collision::new(
                    t,
                    Vec3::new(delta.x, 0.0, delta.z),
                    coll_type
                ));
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
                return Some(Collision::new(
                    t,
                    Vec3::new(delta.x, delta.y, 0.0),
                    CollisionType::Wall
                ));
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
            return Some(Collision::new(
                coll_2d.t,
                Vec3::new(coll_2d.velocity.x, delta.y, coll_2d.velocity.y),
                CollisionType::Wall
            ))
        }
        None
    };

    // Left collision
    // if delta.x > 0.0 {
    //     let coll = x_collision(ter_bounds.min.x - cyl.radius);
    //     if coll.is_some() {
    //         return coll;
    //     }
    // }

    // Right collision
    // if delta.x < 0.0 {
    //     let coll = x_collision(ter_bounds.max.x + cyl.radius);
    //     if coll.is_some() {
    //         return coll;
    //     }
    // }

    // Bottom collision
    // if delta.y > 0.0 {
    //     let coll = bottom_collision(ter_bounds.min.y - cyl.half_height, CollisionType::Ceiling);
    //     if coll.is_some() {
    //         return coll;
    //     }
    // }

    // Top collision
    let coll = top_collision();
    if coll.is_some() {
        return coll;
    }

    // Far collision
    // if delta.z > 0.0 {
    //     let coll = z_collision(ter_bounds.min.z - cyl.radius);
    //     if coll.is_some() {
    //         return coll;
    //     }
    // }

    // Far/left corner collision
    // let coll = edge_collision(Vec2::new(ter_bounds.min.x, ter_bounds.min.z));
    // if coll.is_some() {
    //     return coll;
    // }

    // Far/right corner collision
    // let coll = edge_collision(Vec2::new(ter_bounds.max.x, ter_bounds.min.z));
    // if coll.is_some() {
    //     return coll;
    // }

    // Near/left corner collision
    // let coll = edge_collision(Vec2::new(ter_bounds.min.x, ter_bounds.max.z));
    // if coll.is_some() {
    //     return coll;
    // }

    // Near/right corner collision
    // let coll = edge_collision(Vec2::new(ter_bounds.max.x, ter_bounds.max.z));
    // if coll.is_some() {
    //     return coll;
    // }

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

fn compute_vel(a1: Vec2, b1: Vec2, a2: Vec2, b2: Vec2) -> Vec2 {
    let diff1 = a1 - b1;
    let diff2 = a2 - b2;
    let travel_dir = if diff1.dot(Vec2::X) >= 0.0 {
        -diff2.normalize()
    }
    else {
        diff2.normalize()
    };
    let v = diff1.normalize();
    println!("V: {}", v);
    println!("Y: {}", Vec2::Y);
    println!("dot: {}", v.dot(Vec2::Y));
    let resistance = 1.0 - v.dot(Vec2::Y).abs();
    let vel = diff1.length();
    vel * travel_dir * resistance
}

fn is_ccw(p1: Vec2, p2: Vec2, p3: Vec2) -> bool {
    let val = (p2.y - p1.y) * (p3.x - p2.x)
              - (p2.x - p1.x) * (p3.y - p2.y);
    val <= 0.0
}

fn intersect(a1: Vec2, b1: Vec2, mut a2: Vec2, mut b2: Vec2) -> Option<Collision2D> {

    // Ensures that a2 is to the right of b2
    if b2.x > a2.x {
        std::mem::swap(&mut a2, &mut b2);
    }

    // Ignores case where a1 -> b1 is coming from underneath a2 -> b2
    if !is_ccw(a1, b2, a2) {
        return None;
    }

    // Checks pathological cases
    const EPSILON: f32 = 0.001;
    if a1 == b1 { return None }
    if a2 == b2 { return None }
    if float_eq(a1.x, b1.x, EPSILON) {
        let (slope2, inter2) = slope_intercept_of(a2, b2);
        if a2.x == b2.x {
            return None;
        }
        let y = slope2*a1.x + inter2;
        let t = (y - a1.y) / (b1.y - a1.y);
        if t >= 0.0 && t <= 1.0 {
            return Some(Collision2D {
                t,
                velocity: Vec2::ZERO
            });
        }
        return None;
    }
    else if float_eq(a2.x, b2.x, EPSILON) {
        if a1.x == b1.x {
            return None;
        }
        let t = (a2.x - a1.x) / (b1.x - a1.x);
        if t >= 0.0 && t <= 1.0 {
            return Some(Collision2D {
                t,
                velocity: Vec2::new(0.0, b1.y - a1.y)
            });
        }
        return None;
    }

    // Normal slope intersection
    let (slope1, inter1) = slope_intercept_of(a1, b1);
    let (slope2, inter2) = slope_intercept_of(a2, b2);
    let x = (inter2 - inter1) / (slope1 - slope2);
    let t = (x - a1.x) / (b1.x - a1.x);
    if between(a1.x, b1.x, x) && between(a2.x, b2.x, x) {
        Some(Collision2D {
            t,
            velocity: compute_vel(a1, b1, a2, b2)
        })
    }
    else {
        None
    }
}

#[test]
fn test_intersect_1() {
    let a1 = Vec2::new(3.0, 4.0);
    let b1 = Vec2::new(2.0, 1.0);
    let a2 = Vec2::new(1.0, 3.0);
    let b2 = Vec2::new(3.0, 2.0);
    let intersection = intersect(a1, b1, a2, b2);

    let expected = Some(Collision2D {
        t: 0.57142854,
        velocity: Vec2::new(-0.14514565, 0.07257283)
    });
    assert_eq!(expected, intersection);

    let intersection = intersect(a1, b1, b2, a2);
    assert_eq!(expected, intersection);
}


#[test]
fn test_intersect_2() {
    let a1 = Vec2::new(2.0, 4.0);
    let b1 = Vec2::new(3.0, 1.0);
    let a2 = Vec2::new(1.0, 3.0);
    let b2 = Vec2::new(3.0, 2.0);
    let intersection = intersect(a1, b1, a2, b2);

    let expected = Some(Collision2D {
        t: 0.5999999,
        velocity: Vec2::new(0.14514565, -0.07257283)
    });
    assert_eq!(expected, intersection);

    let intersection = intersect(a1, b1, b2, a2);
    assert_eq!(expected, intersection);
}

#[test]
fn test_intersect_3() {
    let a1 = Vec2::new(4.0, 4.0);
    let b1 = Vec2::new(0.0, 0.0);
    let a2 = Vec2::new(4.0, 0.0);
    let b2 = Vec2::new(0.0, 4.0);
    let intersection = intersect(a1, b1, a2, b2);
    assert_eq!(
        Some(Collision2D {
            t: 0.5,
            velocity: Vec2::new(-1.1715728, 1.1715728)
        }),
        intersection
    );
}

#[test]
fn test_intersect_4() {
    let a1 = Vec2::new(4.0, 4.0);
    let b1 = Vec2::new(0.0, 0.0);
    let a2 = Vec2::new(0.0, 4.0);
    let b2 = Vec2::new(-4.0, -8.0);
    let intersection = intersect(a1, b1, a2, b2);
    assert_eq!(
        None,
        intersection
    );
}

#[test]
fn test_intersect_5() {
    let a1 = Vec2::new(2.0, 2.0);
    let b1 = Vec2::new(0.0, 0.0);
    let a2 = Vec2::new(0.0, 0.0);
    let b2 = Vec2::new(-4.0, 4.0);
    let intersection = intersect(a1, b1, a2, b2);
    assert_eq!(
        Some(Collision2D {
            t: 1.0,
            velocity: Vec2::new(-0.5857864, 0.5857864)
        }),
        intersection
    );
}