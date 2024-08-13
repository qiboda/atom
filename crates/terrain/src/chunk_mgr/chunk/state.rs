use autoincrement::{AutoIncrement, Incremental};
use bevy::math::bounding::Aabb3d;
use bevy::{prelude::*, render::extract_component::ExtractComponent};
use bitflags::bitflags;
use strum::EnumCount;

use crate::isosurface::dc::gpu_dc::buffer_cache::TerrainChunkVertexInfo;
use crate::lod::lod_octree::{LodOctreeLevelType, TerrainLodOctreeNode};
use crate::lod::morton_code::MortonCode;
use crate::tables::{AxisType, SubNodeIndex};

#[derive(Debug, Component, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord, ExtractComponent)]
pub struct TerrainChunkState(u8);

bitflags! {
    // 计算shader根据这个决定是否创建新的mesh
    impl TerrainChunkState: u8 {
        const DONE = 0x0;
        const CREATE_MAIN_MESH = 0x1;
        /// TODO 细化为seam的x,y,z三个方向
        const CREATE_SEAM_MESH = 0x2;
    }
}

#[derive(Incremental, PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub struct SeamMeshId(u64);

#[derive(Component, Debug, Clone)]
pub struct SeamMeshIdGenerator(AutoIncrement<SeamMeshId>);

impl Default for SeamMeshIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl SeamMeshIdGenerator {
    pub fn new() -> Self {
        Self(SeamMeshId::init())
    }

    /// 递增id，返回旧的id。
    pub fn pull(&mut self) -> SeamMeshId {
        self.0.pull()
    }

    // 递增id，并返回一个递增了的id。
    pub fn gen(&mut self) -> SeamMeshId {
        self.pull();
        self.current()
    }

    pub fn current(&self) -> SeamMeshId {
        self.0.current()
    }
}

#[derive(
    Debug, PartialEq, Eq, Hash, Clone, Copy, Component, Default, Deref, DerefMut, ExtractComponent,
)]
pub struct TerrainChunkAddress(pub MortonCode);

impl TerrainChunkAddress {
    pub fn new(address: MortonCode) -> Self {
        Self(address)
    }
}

impl From<&MortonCode> for TerrainChunkAddress {
    fn from(value: &MortonCode) -> Self {
        Self(*value)
    }
}

impl From<MortonCode> for TerrainChunkAddress {
    fn from(value: MortonCode) -> Self {
        Self(value)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Component, Default, ExtractComponent)]
pub struct TerrainChunkNeighborLodNodes {
    pub nodes: [Vec<TerrainLodOctreeNode>; SubNodeIndex::COUNT],
}

#[derive(Debug, Clone, Component, Default, ExtractComponent)]
pub struct TerrainChunkBorderVertices {
    pub vertices: Vec<TerrainChunkVertexInfo>,
    pub vertices_aabb: Vec<Aabb3d>,
}

// 相对lod，0，1, 2, 3, 4
// 值越大，表示深度越浅。
#[derive(
    Debug, PartialEq, Eq, Hash, Clone, Copy, Component, Default, Deref, DerefMut, ExtractComponent,
)]
pub struct TerrainChunkSeamLod(pub [[LodOctreeLevelType; 8]; 8]);

impl TerrainChunkSeamLod {
    pub fn get_lod(&self, subnode_index: SubNodeIndex) -> &[LodOctreeLevelType; 8] {
        &self.0[subnode_index.to_index()]
    }

    pub fn to_uniform_buffer_array(&self) -> [UVec4; 16] {
        let mut array = [UVec4::ZERO; 16];
        for i in 0..8 {
            let lod = self.0[i].map(|x| x as u32);
            for j in 0..2 {
                array[i * 2 + j] = UVec4::from_slice(&lod[j * 4..(j + 1) * 4]);
            }
        }
        array
    }

    pub fn get_max_lod(&self) -> u8 {
        *self.iter().flatten().max().unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TerrainChunkGPUSeamMeshEntities {
    pub right_seam_mesh: Option<Entity>,
    pub top_seam_mesh: Option<Entity>,
    pub front_seam_mesh: Option<Entity>,
}

impl TerrainChunkGPUSeamMeshEntities {
    pub fn despawn_recursive(&self, commands: &mut Commands) {
        if let Some(entity) = self.right_seam_mesh {
            commands.entity(entity).despawn_recursive();
        }
        if let Some(entity) = self.top_seam_mesh {
            commands.entity(entity).despawn_recursive();
        }
        if let Some(entity) = self.front_seam_mesh {
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TerrainChunkCPUSeamMeshEntities {
    pub seam_mesh: Option<Entity>,
}

impl TerrainChunkCPUSeamMeshEntities {
    pub fn despawn_recursive(&self, commands: &mut Commands) {
        if let Some(entity) = self.seam_mesh {
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerrainChunkSeamMeshEntities {
    GPU(TerrainChunkGPUSeamMeshEntities),
    CPU(TerrainChunkCPUSeamMeshEntities),
}

impl TerrainChunkSeamMeshEntities {
    pub fn despawn_recursive(&self, commands: &mut Commands) {
        match self {
            TerrainChunkSeamMeshEntities::GPU(seam_mesh_entities) => {
                seam_mesh_entities.despawn_recursive(commands);
            }
            TerrainChunkSeamMeshEntities::CPU(seam_mesh_entities) => {
                seam_mesh_entities.despawn_recursive(commands);
            }
        }
    }

    pub fn set_cpu_seam_mesh(&mut self, mesh: Entity) {
        if let TerrainChunkSeamMeshEntities::CPU(seam_mesh_entities) = self {
            seam_mesh_entities.seam_mesh = Some(mesh);
        }
    }

    pub fn set_gpu_seam_mesh(&mut self, mesh: Entity, axis: AxisType) {
        if let TerrainChunkSeamMeshEntities::GPU(seam_mesh_entities) = self {
            match axis {
                AxisType::XAxis => seam_mesh_entities.right_seam_mesh = Some(mesh),
                AxisType::YAxis => seam_mesh_entities.top_seam_mesh = Some(mesh),
                AxisType::ZAxis => seam_mesh_entities.front_seam_mesh = Some(mesh),
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Component)]
pub struct TerrainChunkMeshEntities {
    pub main_mesh: Option<Entity>,
    pub seam_mesh: TerrainChunkSeamMeshEntities,
}

impl Default for TerrainChunkMeshEntities {
    fn default() -> Self {
        Self {
            main_mesh: Default::default(),
            seam_mesh: TerrainChunkSeamMeshEntities::CPU(TerrainChunkCPUSeamMeshEntities {
                seam_mesh: Default::default(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TerrainChunkSeamLod;

    #[test]
    fn test_terrain_chunk_lod_to_uniform_buffer_array() {
        let lod = TerrainChunkSeamLod([
            [0, 1, 2, 3, 4, 5, 6, 7],
            [10, 11, 12, 13, 14, 15, 16, 17],
            [20, 21, 22, 23, 24, 25, 26, 27],
            [30, 31, 32, 33, 34, 35, 36, 37],
            [40, 41, 42, 43, 44, 45, 46, 47],
            [50, 51, 52, 53, 54, 55, 56, 57],
            [60, 61, 62, 63, 64, 65, 66, 67],
            [70, 71, 72, 73, 74, 75, 76, 77],
        ]);

        let arr = lod.to_uniform_buffer_array();
        assert_eq!(arr[8].x, 40);
        assert_eq!(arr[9].x, 44);
        assert_eq!(arr[0].x, 0);
        assert_eq!(arr[1].z, 6);
    }
}
