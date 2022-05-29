use std::fmt::Debug;

use bevy::{prelude::*, math::Vec3Swizzles};

use crate::physics::TerrainPiece;

use super::{Terrain, Coords, TerrainPieceRef};

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
pub struct PieceCollider {
    pub piece: TerrainPiece,
    pub position: Vec3,
    pub size: Vec3
}

impl PieceCollider {

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
            if t >= 0.0 && t <= 1.0 {
                let lerped_center = cyl.center + delta * t;
                let lerped_bottom = lerped_center.y - cyl.half_height;
                let lerped_top = lerped_center.y + cyl.half_height;
                let in_yz_bounds =
                    lerped_center.z > ter_bounds.min.z &&
                    lerped_center.z < ter_bounds.max.z &&
                    lerped_bottom < ter_bounds.max.y &&
                    lerped_top > ter_bounds.min.y;
                if in_yz_bounds {
                    let velocity =
                        if t < 1.0 - T_EPSILON {
                        let vel_2d = (next_cyl_center.yz() - lerped_center.yz()) / (1.0 - t);
                            Vec3::new(0.0, vel_2d.x, vel_2d.y)
                        }
                        else { Vec3::new(0.0, delta.y, delta.z) };
                    return Some(Collision {
                        t,
                        velocity,
                    });
                }
            }
            None
        };

        // Collision code for bottom and top sides of this cuboid
        let y_collision = |ter_edge: f32| {
            let t = (ter_edge - cyl.center.y) / delta.y;
            if t >= 0.0 && t <= 1.0 {
                let lerped_center = cyl.center + delta * t;
                let lerped_center_xz = lerped_center.xz();
                let in_xz_bounds =
                    Rect {
                        min: Vec2::new(ter_bounds.min.x - cyl.radius, ter_bounds.min.z),
                        max: Vec2::new(ter_bounds.max.x + cyl.radius, ter_bounds.max.z)
                    }.contains_point(lerped_center_xz) ||
                    Rect {
                        min: Vec2::new(ter_bounds.min.x, ter_bounds.min.z - cyl.radius),
                        max: Vec2::new(ter_bounds.max.x, ter_bounds.max.z + cyl.radius)
                    }.contains_point(lerped_center_xz) ||
                    Circle {
                        center: ter_bounds.min.xz(),
                        radius: cyl.radius
                    }.contains_point(lerped_center_xz) ||
                    Circle {
                        center: Vec2::new(ter_bounds.max.x, ter_bounds.min.z),
                        radius: cyl.radius
                    }.contains_point(lerped_center_xz) ||
                    Circle {
                        center: Vec2::new(ter_bounds.min.x, ter_bounds.max.z),
                        radius: cyl.radius
                    }.contains_point(lerped_center_xz) ||
                    Circle {
                        center: ter_bounds.max.xz(),
                        radius: cyl.radius
                    }.contains_point(lerped_center_xz);
                if in_xz_bounds {
                    let velocity =
                        if t < 1.0 - T_EPSILON {
                            let vel_2d = (next_cyl_center.xz() - lerped_center_xz) / (1.0 - t);
                            Vec3::new(vel_2d.x, 0.0, vel_2d.y)
                        }
                        else { Vec3::new(delta.x, 0.0, delta.z) };
                    return Some(Collision {
                        t,
                        velocity,
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
                    let velocity =
                        if t < 1.0 - T_EPSILON {
                            let vel_2d = (next_cyl_center.xy() - lerped_center.xy()) / (1.0 - t);
                            Vec3::new(vel_2d.x, vel_2d.y, 0.0)
                        }
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

        // Bottom collision
        if delta.y > 0.0 {
            let coll = y_collision(ter_bounds.min.y - cyl.half_height);
            if coll.is_some() {
                return coll;
            }
        }
        

        // Top collision
        if delta.y < 0.0 {
            let coll = y_collision(ter_bounds.max.y + cyl.half_height);
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

    fn aabb(&self) -> Aabb {
        Aabb { min: self.position, max: self.position + self.size }
    }
}

pub trait TerrainCollider {
    fn collide_with_cylinder(&self, cylinder: &CylinderCollider, delta: Vec3) -> Option<Collision>;
}

impl TerrainCollider for Terrain {

    fn collide_with_cylinder(&self, cylinder: &CylinderCollider, delta: Vec3) -> Option<Collision> {
        let mut closest_coll = None;

        // Determines terrain area to select based on cylinder's size and movement
        let piece_size = self.piece_size();
        let cyl_aabb = moving_cylinder_aabb(cylinder, delta);
        let min = cyl_aabb.min / piece_size;
        let max = cyl_aabb.max / piece_size;
        let min = Coords::new(min.x as i32, min.y as i32, min.z as i32);
        let max = Coords::new(max.x as i32 + 1, max.y as i32 + 1, max.z as i32 + 1);


        // For all terrain pieces in the selection...
        println!("-----");
        for piece_ref in self.iter_pieces(min, max) {
            println!("Piece: {:?}", piece_ref.piece);

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
            let collision = piece_coll.collide_with_cylinder(cylinder, delta);
            closest_coll = closest_collision(collision, closest_coll);
        }
        
        // Returns closest collision
        closest_coll
    }
}

// Computes the Aabb of a moving cylinder
fn moving_cylinder_aabb(cylinder: &CylinderCollider, delta: Vec3) -> Aabb {
    let mut min = Vec3::new(
        cylinder.center.x - cylinder.radius,
        cylinder.center.y - cylinder.half_height,
        cylinder.center.z - cylinder.radius
    );
    let mut max = Vec3::new(
        cylinder.center.x + cylinder.radius,
        cylinder.center.y + cylinder.half_height,
        cylinder.center.z + cylinder.radius
    );
    if delta.x < 0.0 {
        min.x += delta.x;
        max.x += delta.x;
    }
    if delta.y < 0.0 {
        min.y += delta.y;
        max.y += delta.y;
    }
    if delta.z < 0.0 {
        min.z += delta.z;
        max.z += delta.z;
    }
    Aabb { min, max }
}

fn closest_collision<'a>(a: Option<Collision>, b: Option<Collision>) -> Option<Collision> {
    match a {
        Some(a_coll) => {
            match b {
                Some(b_coll) => {
                    if a_coll.t < b_coll.t { a }
                    else { b }
                }
                None => a
            }
        }
        None => b
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

impl Circle {
    fn contains_point(&self, point: Vec2) -> bool {
        let dist_squared = (point - self.center).length_squared();
        let rad_squared = self.radius*self.radius;
        dist_squared <= rad_squared
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Rect {
    min: Vec2,
    max: Vec2
}

impl Rect {
    fn contains_point(&self, point: Vec2) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y
    }
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
fn test_collide() {

    fn test(cyl: CylinderCollider, coll: PieceCollider, delta: Vec3, expected: Option<Collision>) {
        let collision = coll.collide_with_cylinder(&cyl, delta);
        assert_eq!(expected, collision);
    }

    let terrain_collider = PieceCollider {
        piece: TerrainPiece::Cuboid,
        position: Vec3::new(0.0, 0.0, 0.0),
        size: Vec3::new(10.0, 10.0, 10.0)
    };

    // Left
    test(
        CylinderCollider {
            center: Vec3::new(-15.0, 5.0, 5.0),
            half_height: 5.0,
            radius: 10.0
        },
        terrain_collider,
        Vec3::new(10.0, 0.0, 5.0),
        Some(Collision {
            t: 0.5,
            velocity: Vec3::new(0.0, 0.0, 5.0)
        })
    );

    // Right
    test(
        CylinderCollider {
            center: Vec3::new(25.0, 5.0, 5.0),
            half_height: 5.0,
            radius: 10.0
        },
        terrain_collider,
        Vec3::new(-10.0, 0.0, 5.0),
        Some(Collision {
            t: 0.5,
            velocity: Vec3::new(0.0, 0.0, 5.0)
        })
    );

    // Near
    test(
        CylinderCollider {
            center: Vec3::new(5.0, 5.0, 20.0),
            half_height: 5.0,
            radius: 10.0
        },
        terrain_collider,
        Vec3::new(5.0, 0.0, -15.0),
        Some(Collision {
            t: 0.0,
            velocity: Vec3::new(5.0, 0.0, 0.0)
        })
    );

    // Far
    test(
        CylinderCollider {
            center: Vec3::new(5.0, 5.0, -12.0),
            half_height: 5.0,
            radius: 10.0
        },
        terrain_collider,
        Vec3::new(5.0, 0.0, 15.0),
        Some(Collision {
            t: 0.13333334,
            velocity: Vec3::new(5.0, 0.0, 0.0)
        })
    );

    // Far
    test(
        CylinderCollider {
            center: Vec3::new(5.0, 5.0, -12.0),
            half_height: 5.0,
            radius: 10.0
        },
        terrain_collider,
        Vec3::new(5.0, 0.0, 15.0),
        Some(Collision {
            t: 0.13333334,
            velocity: Vec3::new(5.0, 0.0, 0.0)
        })
    );

    let terrain_collider = PieceCollider {
        piece: TerrainPiece::Cuboid,
        position: Vec3::new(3.0, 0.0, -6.0),
        size: Vec3::new(3.0, 4.0, 3.0)
    };

    // Top
    test(
        CylinderCollider {
            center: Vec3::new(4.0, 6.0, -4.0),
            half_height: 1.0,
            radius: 1.0
        },
        terrain_collider,
        Vec3::new(0.0, -2.0, 0.0),
        Some(Collision {
            t: 0.5,
            velocity: Vec3::new(0.0, 0.0, 0.0)
        })
    );

    // Top 2
    test(
        CylinderCollider {
            center: Vec3::new(2.0, 6.0, -4.0),
            half_height: 1.0,
            radius: 1.0
        },
        terrain_collider,
        Vec3::new(0.0, -2.0, 0.0),
        Some(Collision {
            t: 0.5,
            velocity: Vec3::new(0.0, 0.0, 0.0)
        })
    );

    // Top 3
    let mov = std::f32::consts::FRAC_1_SQRT_2;
    test(
        CylinderCollider {
            center: Vec3::new(3.0-mov, 6.0, -6.0-mov),
            half_height: 1.0,
            radius: 1.0
        },
        terrain_collider,
        Vec3::new(0.0, -2.0, 0.0),
        Some(Collision {
            t: 0.5,
            velocity: Vec3::new(0.0, 0.0, 0.0)
        })
    );

    // Top 4
    let mov = std::f32::consts::FRAC_1_SQRT_2 + 0.00001;
    test(
        CylinderCollider {
            center: Vec3::new(3.0-mov, 6.0, -6.0-mov),
            half_height: 1.0,
            radius: 1.0
        },
        terrain_collider,
        Vec3::new(0.0, -2.0, 0.0),
        None
    );

    // Bottom
    test(
        CylinderCollider {
            center: Vec3::new(4.0, -2.0, -4.0),
            half_height: 1.0,
            radius: 1.0
        },
        terrain_collider,
        Vec3::new(0.0, 2.0, 0.0),
        Some(Collision {
            t: 0.5,
            velocity: Vec3::new(0.0, 0.0, 0.0)
        })
    );

    // Far/left corner
    let mov = 2.0 * std::f32::consts::FRAC_1_SQRT_2;
    test(
        CylinderCollider {
            center: Vec3::new(3.0-mov, 2.0, -6.0-mov),
            half_height: 2.0,
            radius: 1.0
        },
        terrain_collider,
        Vec3::new(mov, 0.0, mov),
        Some(Collision { t: 0.50000006, velocity: Vec3::new(0.0, 0.0, 0.0) })
    );

    // Far/left corner edgecase
    test(
        CylinderCollider {
            center: Vec3::new(3.0, 2.0, -8.0),
            half_height: 2.0,
            radius: 1.0
        },
        terrain_collider,
        Vec3::new(0.0, 0.0, 2.0),
        Some(Collision { t: 0.5, velocity: Vec3::new(0.0, 0.0, 0.0) })
    );

    // Near/right corner
    test(
        CylinderCollider {
            center: Vec3::new(6.0+mov, 2.0, -3.0+mov),
            half_height: 2.0,
            radius: 1.0
        },
        terrain_collider,
        Vec3::new(-mov, 0.0, -mov),
        Some(Collision { t: 0.50000006, velocity: Vec3::new(0.0, 0.0, 0.0) })
    );

    // Missing
    test(
        CylinderCollider {
            center: Vec3::new(-20.0, 15.0, 0.0),
            half_height: 5.0,
            radius: 10.0
        },
        PieceCollider {
            piece: TerrainPiece::Cuboid,
            position: Vec3::new(0.0, 0.0, 0.0),
            size: Vec3::new(10.0, 10.0, 10.0)
        },
        Vec3::new(20.0, 0.0, 5.0),
        None
    );
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