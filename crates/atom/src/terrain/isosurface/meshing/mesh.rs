use bevy::{
    prelude::{
        debug, default, info, Assets, BuildChildren, Color, Commands, Component, Entity, Mesh,
        PbrBundle, Query, ResMut, StandardMaterial, Transform, UVec3, Vec3,
    },
    render::render_resource::PrimitiveTopology,
    utils::HashMap,
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

impl From<MeshCache> for Mesh {
    fn from(mesh_cache: MeshCache) -> Self {
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

pub fn create_mesh(
    mut commands: Commands,
    mut mesh_cache: Query<(Entity, &MeshCache, &mut IsosurfaceExtractionState)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mesh_cache, mut state) in mesh_cache.iter_mut() {
        if let IsosurfaceExtractionState::MeshCreate = *state {
            info!("create_mesh");
            let id = commands
                .spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(mesh_cache.clone())),
                    material: materials.add(StandardMaterial {
                        base_color: Color::BLUE,
                        double_sided: false,
                        // cull_mode: None,
                        ..default()
                    }),
                    transform: Transform::from_translation(Vec3::splat(0.0)),
                    ..Default::default()
                })
                .id();

            let mut terrian = commands.get_entity(entity).unwrap();
            terrian.add_child(id);

            *state = IsosurfaceExtractionState::Done;
        }
    }
}
