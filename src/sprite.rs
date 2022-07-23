use std::collections::HashMap;

use bevy::math::Vec3A;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use bevy::render::{render_resource::PrimitiveTopology};
use bevy::render::mesh::{VertexAttributeValues, Indices};

/// Batch drawing stage
const DRAW_BATCHES_STAGE: &str = "draw_batches";

/// Plugin dedicated to rendering plain sprites in 3D
pub struct SpritePlugin;
impl Plugin for SpritePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<BatchRenderer>()
            .add_stage_after(CoreStage::PostUpdate, DRAW_BATCHES_STAGE, SystemStage::single_threaded())
            .add_system_to_stage(DRAW_BATCHES_STAGE, draw_sprites)
        ;
    }
}

/// Component representing a 3D sprite quad to be drawn
#[derive(Component, Debug, Clone, Default)]
pub struct Sprite3D {
    pub size: Vec2,
    pub region: Region,
    pub offset: Vec3
}

impl Sprite3D {
    pub fn new(size: Vec2, region: Region) -> Self {
        Sprite3D { size, region, offset: Vec3::ZERO }
    }
}

/// Bundle for a Sprite3D + auxiliary components
#[derive(Bundle, Clone, Debug)]
pub struct Sprite3DBundle {

    /// Sprite region itself
    pub sprite: Sprite3D,

    /// Material the sprite samples from
    pub material: Handle<StandardMaterial>,
    
    /// Position, scale and rotation of the sprite to use when drawing
    pub transform: Transform,

    /// Global transform used for parent/child relationships. Do not touch.
    pub global_transform: GlobalTransform
}

/// Rectangular region used for UV mapping
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Region {
    pub min: Vec2,
    pub max: Vec2,
}

/// Place to buffer sprite commands and deposit them to entities with a mesh
pub struct MeshInfo {
    pub entity: Entity,
    pub mesh_handle: Handle<Mesh>,
    pub draw_quad_commands: Vec<DrawQuadCommand>
}

/// Resource used for drawing textured sprites in a 3D space
#[derive(Default)]
pub struct BatchRenderer {
    /// Mapping of material id to mesh info
    pub mesh_infos: HashMap<Handle<StandardMaterial>, MeshInfo>
}

impl BatchRenderer {

    pub fn draw_quads(
        &mut self,
        draw_quad_commands: &[DrawQuadCommand],
        meshes: &mut Assets<Mesh>,
        mat_handle: &Handle<StandardMaterial>,
        commands: &mut Commands
    ){

        // Gets/creates mesh info associated with material id
        let mesh_info = self.mesh_infos
            .entry(mat_handle.clone_weak())
            .or_insert_with(|| {
                let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<[f32; 3]>::new());
                mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, Vec::<[f32; 3]>::new());
                mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, Vec::<[f32; 2]>::new());
                mesh.set_indices(Some(Indices::U32(Vec::new())));
                let mesh_handle = meshes.add(mesh);
                let entity = commands
                    .spawn_bundle(PbrBundle {
                        mesh: mesh_handle.clone(),
                        material: mat_handle.clone_weak(),
                        ..Default::default()
                    })
                    .insert(Aabb {
                        center: Vec3A::ZERO,
                        half_extents: Vec3A::new(f32::MAX, f32::MAX, f32::MAX)
                    })
                    .id();
                MeshInfo {
                    mesh_handle,
                    entity,
                    draw_quad_commands: Vec::new()
                }
            });
        
        // Buffers draw command for later execution
        for command in draw_quad_commands {
            mesh_info.draw_quad_commands.push(*command);
        }
    }

    pub fn flush(&mut self, meshes: &mut Assets<Mesh>) {

        // For akll mesh infos
        for mesh_info in self.mesh_infos.values_mut() {

            // Get mesh
            let mesh = meshes.get_mut(&mesh_info.mesh_handle).unwrap();

            // Write position data
            let pos_data = match mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap() {
                VertexAttributeValues::Float32x3(vec) => vec,
                _ => panic!("Position data not found")
            };
            pos_data.clear();
            for command in &mesh_info.draw_quad_commands {
                let t = command.transform;
                let v1 = t.mul_vec3(Vec3::ZERO).to_array();
                let v2 = t.mul_vec3(Vec3::new(command.size.x, 0.0, 0.0)).to_array();
                let v3 = t.mul_vec3(Vec3::new(command.size.x, command.size.y, 0.0)).to_array();
                let v4 = t.mul_vec3(Vec3::new(0.0, command.size.y, 0.0)).to_array();
                pos_data.extend_from_slice(&[v1, v2, v3, v4]);
            }

            // Writes normal data
            let normal_data = match mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL).unwrap() {
                VertexAttributeValues::Float32x3(vec) => vec,
                _ => panic!("Position data not found")
            };
            normal_data.clear();
            for _ in 0..mesh_info.draw_quad_commands.len() * 4 {
                normal_data.push([0.0, 0.0, 1.0]);
            }

            // Writes UV data
            let uv_data = match mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0).unwrap() {
                VertexAttributeValues::Float32x2(vec) => vec,
                _ => panic!("Position data not found")
            };
            uv_data.clear();
            for command in &mesh_info.draw_quad_commands {
                let reg = command.uv_region;
                let uv1 = [reg.min.x, reg.max.y];
                let uv2 = [reg.max.x, reg.max.y];
                let uv3 = [reg.max.x, reg.min.y];
                let uv4 = [reg.min.x, reg.min.y];
                uv_data.extend_from_slice(&[uv1, uv2, uv3, uv4]);
            }

            // Writes index data
            let indices = match mesh.indices_mut().unwrap() {
                Indices::U32(vec) => vec,
                _ => panic!("Unexpected index type")
            };
            indices.clear();
            for i in 0..mesh_info.draw_quad_commands.len() {
                let i = i as u32;
                indices.extend_from_slice(&[i, i+1, i+2, i+2, i+3, i]);
            }

            // Clears draw commands
            mesh_info.draw_quad_commands.clear();
        }
    }

    // Removes entries belonging to materials that are unloaded
    pub fn refresh(
        &mut self,
        materials: &Assets<StandardMaterial>,
        commands: &mut Commands
    ) {
        self.mesh_infos.retain(|mat_handle, mesh_info| {
            let is_loaded = materials.contains(mat_handle);
            if !is_loaded {
                commands.entity(mesh_info.entity).despawn();
            }
            is_loaded
        });
    }
}

/// Command to send to a MeshInfo for drawing textured quads
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct DrawQuadCommand {
    pub transform: GlobalTransform,
    pub size: Vec2,
    pub uv_region: Region
}

/// System that collects sprite information from entities, and draws them to the proper mesh entities
fn draw_sprites(
    sprite_query: Query<(&Sprite3D, &Handle<StandardMaterial>, &GlobalTransform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    mut batch_renderer: ResMut<BatchRenderer>,
    mut commands: Commands
) {
    log::debug!("(SYSTEM) draw_sprites");

    // Clears mesh handles that are no longer loaded and despawns their entities
    batch_renderer.refresh(&materials, &mut commands);

    // For all sprites...
    for (sprite, mat_handle, global) in sprite_query.iter() {

        // Buffers draw command for sprite
        let trans = Transform::from_translation(sprite.offset);
        let global = (*global).mul_transform(trans);
        let draw_command = DrawQuadCommand {
            transform: global,
            size: sprite.size,
            uv_region: sprite.region
        };
        batch_renderer.draw_quads(&[draw_command], &mut meshes, &mat_handle, &mut commands);
    }

    // Executes buffered draw commands
    batch_renderer.flush(&mut meshes);
}