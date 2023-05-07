use bevy::{prelude::*, utils::HashMap};

#[derive(Debug, Resource, Default)]
pub struct TerrainCubeData {
    pub mesh: Option<Handle<Mesh>>,
    pub material: Option<Handle<StandardMaterial>>,
}

#[derive(Debug, Resource, Default)]
pub struct TerrainVoxelCubeMap {
    pub voxel_cube_map: HashMap<Entity, Entity>,
    pub cube_voxel_map: HashMap<Entity, Entity>,
}

impl TerrainVoxelCubeMap {
    pub fn get_voxel_cube(&self, voxel_entity: Entity) -> Option<&Entity> {
        self.voxel_cube_map.get(&voxel_entity)
    }

    pub fn get_cube_voxel(&self, cube_entity: Entity) -> Option<&Entity> {
        self.cube_voxel_map.get(&cube_entity)
    }

    pub fn set_voxel_cube(&mut self, voxel_entity: Entity, cube_entity: Entity) {
        self.voxel_cube_map.insert(voxel_entity, cube_entity);
        self.cube_voxel_map.insert(cube_entity, voxel_entity);
    }

    pub fn remove_voxel_cube(&mut self, voxel_entity: Entity) {
        if let Some(cube_entity) = self.voxel_cube_map.remove(&voxel_entity) {
            self.cube_voxel_map.remove(&cube_entity);
        }
    }
}
