use std::fmt::Debug;

use bevy::{prelude::*, math::Vec3Swizzles};

use crate::physics::TerrainPiece;

const T_EPSILON: f32 = 0.001;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Collision {
    pub t: f32,
    pub velocity: Vec3
}

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
/// Collider of a [`TerrainPiece`].
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TerrainCollider {
    pub piece: TerrainPiece,
    pub position: Vec3,
    pub size: Vec3
}

impl TerrainCollider {

    /// Collides a terrain piece with a cylinder's movement
    pub fn collide_with_cylinder(&self, cyl: &CylinderCollider, delta: Vec3) -> Option<Collision> {
        match self.piece {
            TerrainPiece::Cuboid => self.collide_cuboid_with_cylinder(cyl, delta),
            //TerrainPiece::Slope => self.collide_slope_with_cylinder(cyl, delta),
            _ => None
        }
    }

    fn collide_cuboid_with_cylinder(&self, cyl: &CylinderCollider, delta: Vec3) -> Option<Collision> {

        // Terrain bounds
        let ter_bounds = self.aabb();

        // Unpacks movement
        let next_cyl_center = cyl.center + delta;

        // Collision code for left and right sides of this cuboid
        let x_collision = |ter_edge: f32| {
            let t = (ter_edge - cyl.center.x) / delta.x;
            if t >= 0.0 && t < 1.0 {
                let lerped_center = cyl.center + delta * t;
                let lerped_bottom = lerped_center.y - cyl.half_height;
                let lerped_top = lerped_center.y + cyl.half_height;
                let in_yz_bounds =
                    lerped_center.z > ter_bounds.min.z &&
                    lerped_center.z < ter_bounds.max.z &&
                    lerped_bottom < ter_bounds.max.y &&
                    lerped_top > ter_bounds.min.y;
                if in_yz_bounds {
                    let mut final_center = next_cyl_center;
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
                    let mut final_center = next_cyl_center;
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

        // Collision of vertical line with cylinder
        let edge_collision = |edge: Vec2| -> Option<Collision> {
            let cir = Circle {
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
                    velocity: Vec3::new(coll_2d.velocity.x, delta.y, coll_2d.velocity.y)
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

        // Far/left corner collision
        let coll = edge_collision(Vec2::new(ter_bounds.max.x, ter_bounds.min.z));
        if coll.is_some() {
            return coll;
        }

        // Far/left corner collision
        let coll = edge_collision(Vec2::new(ter_bounds.min.x, ter_bounds.max.z));
        if coll.is_some() {
            return coll;
        }

        // Far/left corner collision
        let coll = edge_collision(Vec2::new(ter_bounds.max.x, ter_bounds.max.z));
        if coll.is_some() {
            return coll;
        }

        // Default
        None
    }

    fn aabb(&self) -> Aabb {
        Aabb { min: self.position, max: self.position + self.size }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Collision2D {
    pub t: f32,
    pub velocity: Vec2
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Circle {
    center: Vec2,
    radius: f32
}

fn collide_line_with_circle(src: Vec2, dest: Vec2, circle: Circle) -> Option<Collision2D> {

    // Gets closest point on line "d"
    let a = src;
    let c = circle.center;
    let ca = c - a;
    let rad_squared = circle.radius * circle.radius;
    if ca.length_squared() < rad_squared {
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
    if t < 0.0 || t > 1.0 {
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
#[test]
fn collide_left() {
    let cyl = CylinderCollider {
        center: Vec3::new(-15.0, 5.0, 5.0),
        half_height: 5.0,
        radius: 10.0
    };
    let coll = TerrainCollider {
        piece: TerrainPiece::Cuboid,
        position: Vec3::new(0.0, 0.0, 0.0),
        size: Vec3::new(10.0, 10.0, 10.0)
    };
    let delta = Vec3::new(10.0, 0.0, 5.0);
    let collision = coll.collide_with_cylinder(&cyl, delta);
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
    let cyl = CylinderCollider {
        center: Vec3::new(25.0, 5.0, 5.0),
        half_height: 5.0,
        radius: 10.0
    };
    let coll = TerrainCollider {
        piece: TerrainPiece::Cuboid,
        position: Vec3::new(0.0, 0.0, 0.0),
        size: Vec3::new(10.0, 10.0, 10.0)
    };
    let delta = Vec3::new(-10.0, 0.0, 5.0);
    let collision = coll.collide_with_cylinder(&cyl, delta);
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
    let cyl = CylinderCollider {
        center: Vec3::new(5.0, 5.0, 20.0),
        half_height: 5.0,
        radius: 10.0
    };
    let coll = TerrainCollider {
        piece: TerrainPiece::Cuboid,
        position: Vec3::new(0.0, 0.0, 0.0),
        size: Vec3::new(10.0, 10.0, 10.0)
    };
    let delta = Vec3::new(5.0, 0.0, -15.0);
    let collision = coll.collide_with_cylinder(&cyl, delta);
    assert_eq!(
        Some(Collision {
            t: 0.0,
            velocity: Vec3::new(5.0, 0.0, 0.0)
        }),
        collision
    );
}

#[test]
fn collide_far() {
    let cyl = CylinderCollider {
        center: Vec3::new(5.0, 5.0, -12.0),
        half_height: 5.0,
        radius: 10.0
    };
    let coll = TerrainCollider {
        piece: TerrainPiece::Cuboid,
        position: Vec3::new(0.0, 0.0, 0.0),
        size: Vec3::new(10.0, 10.0, 10.0)
    };
    let delta = Vec3::new(5.0, 0.0, 15.0);
    let collision = coll.collide_with_cylinder(&cyl, delta);
    assert_eq!(
        Some(Collision {
            t: 0.13333334,
            velocity: Vec3::new(5.0, 0.0, 0.0)
        }),
        collision
    );
}

#[test]
fn collide_missing() {
    let cyl = CylinderCollider {
        center: Vec3::new(-20.0, 15.0, 0.0),
        half_height: 5.0,
        radius: 10.0
    };
    let coll = TerrainCollider {
        piece: TerrainPiece::Cuboid,
        position: Vec3::new(0.0, 0.0, 0.0),
        size: Vec3::new(10.0, 10.0, 10.0)
    };
    let delta = Vec3::new(20.0, 0.0, 5.0);
    let collision = coll.collide_with_cylinder(&cyl, delta);
    assert_eq!(None, collision);
}

#[test]
fn test_collide_line_with_circle() {

    fn test(a: Vec2, b: Vec2, circle: Circle, expected: Option<Collision2D>) {
        let collision = collide_line_with_circle(a, b, circle);
        assert_eq!(expected, collision);
    }

    let circle = Circle {
        center: Vec2::new(2.0, 0.0),
        radius: 1.0
    };

    test(
        Vec2::ZERO,
        Vec2::new(2.0, 0.0),
        circle,
        Some(Collision2D {
            t: 0.5,
            velocity: Vec2::ZERO
        })
    );

    test(
        Vec2::new(-5.0, 0.0),
        Vec2::new(-4.0, 1.0),
        circle,
        None
    );

    test(
        Vec2::new(5.0, 0.0),
        Vec2::new(6.0, 1.0),
        circle,
        None
    );

    test(
        Vec2::ZERO,
        Vec2::new(1.0, 1.0),
        circle,
        None
    );

    test(
        Vec2::ZERO,
        Vec2::new(1.0, -1.0),
        circle,
        None
    );

    test(
        Vec2::new(-1.0, 0.0),
        Vec2::ZERO,
        circle,
        None
    );

    test(
        Vec2::new(1.0, 0.0),
        Vec2::new(2.0, 0.0),
        circle,
        Some(Collision2D {
            t: 0.0,
            velocity: Vec2::ZERO
        })
    );

    test(
        Vec2::new(1.1, 0.0),
        Vec2::new(2.0, 0.0),
        circle,
        None
    );

    test(
        Vec2::new(0.0, 0.001),
        Vec2::new(2.0, 0.001),
        circle,
        Some(Collision2D {
            t: 0.50000024,
            velocity: Vec2::new(9.536743e-10, 9.5367375e-7)
        })
    );

    test(
        Vec2::new(0.0, 0.99999),
        Vec2::new(4.0, 0.99999),
        circle,
        Some(Collision2D {
            t: 0.49888122,
            velocity: Vec2::new(3.9820597, 0.01782036)
        })
    );

    test(
        Vec2::new(2.99999, 2.0),
        Vec2::new(2.99999, -2.0),
        circle,
        Some(Collision2D {
            t: 0.49888122,
            velocity: Vec2::new(0.017820578, -3.9820595) }
        )
    );

    test(
        Vec2::new(1.9, 2.0),
        Vec2::new(1.9, -2.0),
        circle,
        Some(Collision2D {
            t: 0.25125313,
            velocity: Vec2::new(-0.019949785, -0.0020050292)
        })
    );

    test(
        Vec2::new(3.0, 2.0),
        Vec2::new(3.0, -2.0),
        circle,
        None
    );
}