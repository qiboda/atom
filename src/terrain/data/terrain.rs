use bevy::{
    prelude::{Component, Entity, GlobalTransform},
    utils::HashMap,
};

use super::coords::TerrainChunkCoord;

#[derive(Debug, Component, Default)]
pub struct TerrainData {
    pub data: HashMap<TerrainChunkCoord, Entity>,
}

impl TerrainData {
    pub fn get_chunk_entity_by_coord(
        &self,
        terrain_chunk_coord: TerrainChunkCoord,
    ) -> Option<&Entity> {
        self.data.get(&terrain_chunk_coord)
    }

    pub fn new() -> TerrainData {
        Self::default()
    }

    // pub fn get_visible_voxels(
    //     &self,
    //     min_graded_coord: &VoxelGradedCoord,
    //     max_graded_coord: &VoxelGradedCoord,
    // ) -> Vec<VisibleVoxel> {
    //     let section_datas = self.get_visible_sections(min_graded_coord, max_graded_coord);
    //
    //     let mut voxel_datas = Vec::new();
    //     for data in section_datas {
    //         if data.data.is_none() {
    //             continue;
    //         }
    //
    //         voxel_datas.append(&mut data.data.unwrap().get_visible_voxels(
    //             min_graded_coord,
    //             max_graded_coord,
    //             &data.visible_direction,
    //         ));
    //     }
    //
    //     voxel_datas
    // }
    //
    // pub fn get_visible_sections(
    //     &self,
    //     min_graded_coord: &VoxelGradedCoord,
    //     max_graded_coord: &VoxelGradedCoord,
    // ) -> Vec<VisibleVoxelSection> {
    //     let visible_chunk_datas = self.get_visible_chunks(min_graded_coord, max_graded_coord);
    //
    //     let mut section_datas = Vec::new();
    //     for data in visible_chunk_datas {
    //         if data.data.is_none() {
    //             continue;
    //         }
    //         section_datas.append(&mut data.data.unwrap().get_visible_sections(
    //             min_graded_coord,
    //             max_graded_coord,
    //             data.visible_direction,
    //         ));
    //     }
    //
    //     section_datas
    // }
    //
    // pub fn get_visible_chunks(
    //     &self,
    //     min_graded_coord: &VoxelGradedCoord,
    //     max_graded_coord: &VoxelGradedCoord,
    // ) -> HashSet<VisibleVoxelChunk> {
    //     let mut chunk_visible_datas = HashSet::new();
    //     for x in min_graded_coord.chunk_coord.x..=max_graded_coord.chunk_coord.x {
    //         for y in min_graded_coord.chunk_coord.y..=max_graded_coord.chunk_coord.y {
    //             for z in min_graded_coord.chunk_coord.z..=max_graded_coord.chunk_coord.z {
    //                 let coord = TerrainChunkCoord { x, y, z };
    //                 chunk_visible_datas.insert(TerrainVisible {
    //                     data: self.data.get(&coord).copied(),
    //                     visible_direction: VisibleDirection {
    //                         x: match (
    //                             x == min_graded_coord.chunk_coord.x,
    //                             x == max_graded_coord.chunk_coord.x,
    //                         ) {
    //                             (true, true) => super::visible::VisibleAxis::Unqiue,
    //                             (true, false) => super::visible::VisibleAxis::Negative,
    //                             (false, true) => super::visible::VisibleAxis::Positive,
    //                             (false, false) => super::visible::VisibleAxis::Full,
    //                         },
    //                         y: match (
    //                             y == min_graded_coord.chunk_coord.y,
    //                             y == max_graded_coord.chunk_coord.y,
    //                         ) {
    //                             (true, true) => super::visible::VisibleAxis::Unqiue,
    //                             (true, false) => super::visible::VisibleAxis::Negative,
    //                             (false, true) => super::visible::VisibleAxis::Positive,
    //                             (false, false) => super::visible::VisibleAxis::Full,
    //                         },
    //                         z: match (
    //                             z == min_graded_coord.chunk_coord.z,
    //                             z == max_graded_coord.chunk_coord.z,
    //                         ) {
    //                             (true, true) => super::visible::VisibleAxis::Unqiue,
    //                             (true, false) => super::visible::VisibleAxis::Negative,
    //                             (false, true) => super::visible::VisibleAxis::Positive,
    //                             (false, false) => super::visible::VisibleAxis::Full,
    //                         },
    //                     },
    //                 });
    //             }
    //         }
    //     }
    //
    //     chunk_visible_datas
    // }
    //
    pub fn update_terrain(&self, _global_transform: &GlobalTransform) {}
}
