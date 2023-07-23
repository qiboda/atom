use bevy::{
    prelude::{
        debug, default, info, Assets, BuildChildren, Color, Commands, Component, Entity, Mesh,
        PbrBundle, Query, ResMut, StandardMaterial, Transform, UVec3, Vec3,
    },
    render::render_resource::PrimitiveTopology,
    utils::HashMap,
};
use bevy_xpbd_3d::{
    parry::{
        math::{Point, Real},
        shape::SharedShape,
    },
    prelude::{Collider, RigidBody},
};

use crate::terrain::isosurface::IsosurfaceExtractionState;

use super::vertex_index::VertexIndices;

#[derive(Debug, Clone, Default, Component)]
pub struct MeshCache {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub indices: Vec<u32>,
    pub vertex_index_data: HashMap<UVec3, VertexIndices>,
}

impl MeshCache {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
            vertex_index_data: HashMap::default(),
        }
    }
}

impl MeshCache {
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    fn check(&self) {
        assert!(
            self.positions.len() > 0
                && self.positions.len() == self.normals.len()
                && self.indices.len() % 3 == 0
        );
    }

    pub fn get_vertice_positions(&self) -> &Vec<Vec3> {
        &self.positions
    }

    pub fn set_vertice_positions(&mut self, positions: Vec<Vec3>) {
        self.positions = positions;
    }

    pub fn get_vertice_normals(&self) -> &Vec<Vec3> {
        &self.normals
    }

    pub fn set_vertice_normals(&mut self, normals: Vec<Vec3>) {
        self.normals = normals;
    }

    pub fn set_indices(&mut self, indices: Vec<u32>) {
        self.indices = indices;
    }

    pub fn get_indices(&self) -> &Vec<u32> {
        &self.indices
    }

    pub fn get_indices_mut(&mut self) -> &mut Vec<u32> {
        &mut self.indices
    }
}

impl From<&MeshCache> for Mesh {
    fn from(mesh_cache: &MeshCache) -> Self {
        mesh_cache.check();
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            mesh_cache.get_vertice_positions().clone(),
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            mesh_cache.get_vertice_normals().clone(),
        );
        debug!("mesh cache from: {:?}", mesh_cache.get_indices());
        mesh.set_indices(Some(bevy::render::mesh::Indices::U32(
            mesh_cache.get_indices().clone(),
        )));
        mesh
    }
}

impl From<&MeshCache> for Collider {
    fn from(mesh_cache: &MeshCache) -> Self {
        mesh_cache.check();

        let mut vertices: Vec<Point<Real>> =
            Vec::with_capacity(mesh_cache.get_vertice_positions().len());
        let mut indices: Vec<[u32; 3]> = Vec::with_capacity(mesh_cache.get_indices().len() / 3);

        mesh_cache
            .get_vertice_positions()
            .iter()
            .for_each(|vertex| {
                vertices.push(Point::from_slice(&[vertex.x, vertex.y, vertex.z]));
            });

        for index in mesh_cache.get_indices().chunks(3) {
            indices.push(index.try_into().unwrap());
        }

        Collider::from(SharedShape::trimesh(vertices, indices))
    }
}

pub fn create_mesh(
    mut commands: Commands,
    mut mesh_cache: Query<(Entity, &MeshCache, &mut IsosurfaceExtractionState)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mesh_cache, mut state) in mesh_cache.iter_mut() {
        if let IsosurfaceExtractionState::MeshCreate = *state {
            info!("create_mesh");
            if mesh_cache.is_empty() == false {
                let id = commands
                    .spawn((
                        PbrBundle {
                            mesh: meshes.add(Mesh::from(mesh_cache)),
                            material: materials.add(StandardMaterial {
                                base_color: Color::BLUE,
                                double_sided: false,
                                // cull_mode: None,
                                ..default()
                            }),
                            transform: Transform::from_translation(Vec3::splat(0.0)),
                            ..Default::default()
                        },
                        RigidBody::Static,
                        Collider::from(mesh_cache),
                    ))
                    .id();

                let mut terrian = commands.get_entity(entity).unwrap();
                terrian.add_child(id);
            }

            *state = IsosurfaceExtractionState::Done;
        }
    }
}
