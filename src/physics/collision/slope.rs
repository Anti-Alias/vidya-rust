use bevy::math::{ Vec2, Vec3, Vec3Swizzles };

use crate::physics::collision::t_in_range;

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
            let in_rect_bounds = || {
                lerped_center.z > ter_bounds.min.z &&
                lerped_center.z < ter_bounds.max.z  &&
                lerped_bottom < ter_bounds.max.y &&
                lerped_top > ter_bounds.min.y
            };
            let under_slope = || {
                let a = Vec2::new(ter_bounds.min.z, ter_bounds.max.y);
                let b = Vec2::new(lerped_center.z - cyl.radius, lerped_center.y - cyl.half_height);
                let c = Vec2::new(ter_bounds.max.z, ter_bounds.min.y);
                is_ccw(a, b, c)
            };
            if in_rect_bounds() && under_slope() {
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
            min: Vec2::new(ter_bounds.min.x - cyl.radius, ter_bounds.min.z - cyl.radius),
            max: Vec2::new(ter_bounds.max.x + cyl.radius, ter_bounds.max.z + cyl.radius)
        }.contains_point(point)
    };

    // Collision code for the top side of this slope
    let slope_collision = || -> Option<Collision> {
        let min = ter_bounds.min;
        let max = ter_bounds.max;
        let a1 = Vec2::new(
            cyl.center.z - cyl.radius,
            cyl.center.y - cyl.half_height
        );
        let b1 = a1 + delta.zy();
        let a2 = Vec2::new(max.z, min.y);
        let b2 = Vec2::new(min.z, max.y);
        let (coll2d, offset2d) = collide2d(a1, b1, a2, b2, Vec2::Y)?;
        let vel_zy = coll2d.velocity;
        let ter_point = cyl.center + delta * coll2d.t;
            if in_xz_bounds(ter_point.xz()) {
                const EPSILON: f32 = 0.01;
                let mut new_a2 = a1 + (b1-a1) * coll2d.t + vel_zy + offset2d;
                new_a2.y += EPSILON;
                return Some(Collision {
                    t: coll2d.t,
                    velocity: Vec3::new(delta.x, vel_zy.y, vel_zy.x),
                    offset: Vec3::new(0.0, offset2d.y + EPSILON, 0.0),
                    typ: CollisionType::Floor
                });
            }
        None
    };

    // Collision code for bottom and top sides of this cuboid
    let top_collision = || {
        let ter_edge = ter_bounds.max.y + cyl.half_height;
        let t = (ter_edge - cyl.center.y) / delta.y;
        if t_in_range(t) {
            let lerped_center = cyl.center + delta * t;
            let lerped_center_xz = lerped_center.xz();
            let in_xz_bounds =
                RectHelper {
                    min: Vec2::new(ter_bounds.min.x - cyl.radius, ter_bounds.min.z - cyl.radius),
                    max: Vec2::new(ter_bounds.max.x + cyl.radius, ter_bounds.min.z + cyl.radius)
                }.contains_point(lerped_center_xz);
            if in_xz_bounds {
                return Some(Collision::new(
                    t,
                    Vec3::new(delta.x, 0.0, delta.z),
                    CollisionType::Floor
                ));
            }
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

    // Slope collision
    let coll = slope_collision();
    if coll.is_some() {
        return coll;
    }

    // Top collision
    if delta.y < 0.0 {
        let coll = top_collision();
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
fn between(mut a: f32, mut b: f32, c: f32, epsilon: f32) -> bool {
    if a > b {
        std::mem::swap(&mut a, &mut b);
    }
    c + epsilon >= a && c - epsilon <= b
}

fn is_ccw(p1: Vec2, p2: Vec2, p3: Vec2) -> bool {
    let val = (p2.y - p1.y) * (p3.x - p2.x)
              - (p2.x - p1.x) * (p3.y - p2.y);
    val <= 0.0
}

fn collide2d(a1: Vec2, b1: Vec2, mut a2: Vec2, mut b2: Vec2, normal: Vec2) -> Option<(Collision2D, Vec2)> {

    // Ensures that a2 is to the right of b2
    if b2.x > a2.x {
        std::mem::swap(&mut a2, &mut b2);
    }

    // Ignores case where a1 -> b1 is coming from underneath a2 -> b2
    if !is_ccw(a1, b2, a2) {
        return None;
    }

    // Checks pathological cases
    if a1 == b1 { return None }
    if a2 == b2 { return None }

    let (slope1, intercept1) = slope_intercept_of(a1, b1);
    let (slope2, intercept2) = slope_intercept_of(a2, b2);

    if slope1.abs() > 70.0 {
        let (slope2, inter2) = slope_intercept_of(a2, b2);
        if a2.x == b2.x {
            return None;
        }
        let y = slope2*a1.x + inter2;
        let t = (y - a1.y) / (b1.y - a1.y);
        if t_in_range(t) {
            let collision = Collision2D {
                t,
                velocity: Vec2::ZERO
            };
            let offset = Vec2::ZERO;
            return Some((collision, offset));
        }
        return None;
    }
    else if slope2.abs() > 70.0 {
        if a1.x == b1.x {
            return None;
        }
        let t = (a2.x - a1.x) / (b1.x - a1.x);
        let collision = Collision2D {
            t,
            velocity: Vec2::new(0.0, b1.y - a1.y)
        };
        let offset = Vec2::ZERO;
        if t_in_range(t) {
            return Some((collision, offset));
        }
        return None;
    }

    // Normal slope intersection
    let inter_x = (intercept2 - intercept1) / (slope1 - slope2);
    let t = (inter_x - a1.x) / (b1.x - a1.x);
    if !t_in_range(t) {
        return None;
    }
    const EP: f32 = 0.01;
    let between_first = between(a1.x, b1.x, inter_x, EP);
    let between_second = between(a2.x, b2.x, inter_x, EP);
    if !(between_first || between_second) {
        return None;
    }

    let inter_y = inter_x*slope2 + intercept2;
    let final_vel_x = b1.x - a1.x;
    let collision = Collision2D {
        t,
        velocity: Vec2::new(final_vel_x, 0.0)
    };
    let final_x = inter_x + final_vel_x;
    let final_y = slope2*final_x + intercept2;
    let offset = Vec2::new(0.0, final_y - inter_y) + normal*Vec2::new(0.01, 0.01);
    Some((collision, offset))
}
