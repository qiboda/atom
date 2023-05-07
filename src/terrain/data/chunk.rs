use bevy::prelude::Component;

#[derive(Debug, Default, Component)]
pub struct TerrainChunkData {
    // pub data: HashMap<TerrainChunkCoord, Terrain>,
    pub loaded: bool,
}

impl TerrainChunkData {
    // pub(crate) fn get_visible_sections(
    //     &self,
    //     min_graded_coord: &VoxelGradedCoord,
    //     max_graded_coord: &VoxelGradedCoord,
    //     visible_direction: VisibleDirection,
    // ) -> Vec<VisibleVoxelSection> {
    //     let mut chunk_datas = Vec::new();
    //
    //     let xrange = match visible_direction.x {
    //         super::visible::VisibleAxis::Unqiue => {
    //             debug_assert!(min_graded_coord.section_coord.x == max_graded_coord.section_coord.x);
    //             min_graded_coord.section_coord.x..(max_graded_coord.section_coord.x + 1)
    //         }
    //         super::visible::VisibleAxis::Positive => 0..max_graded_coord.section_coord.x,
    //         super::visible::VisibleAxis::Negative => {
    //             min_graded_coord.section_coord.x..TERRAIN_CHUNK_SECTION_SIZE
    //         }
    //         super::visible::VisibleAxis::Full => 0..TERRAIN_CHUNK_SECTION_SIZE,
    //     };
    //
    //     let yrange = match visible_direction.y {
    //         super::visible::VisibleAxis::Unqiue => {
    //             min_graded_coord.section_coord.y..(max_graded_coord.section_coord.y + 1)
    //         }
    //         super::visible::VisibleAxis::Positive => 0..max_graded_coord.section_coord.y,
    //         super::visible::VisibleAxis::Negative => {
    //             min_graded_coord.section_coord.y..TERRAIN_CHUNK_SECTION_SIZE
    //         }
    //         super::visible::VisibleAxis::Full => 0..TERRAIN_CHUNK_SECTION_SIZE,
    //     };
    //
    //     let zrange = match visible_direction.z {
    //         super::visible::VisibleAxis::Unqiue => {
    //             min_graded_coord.section_coord.z..(max_graded_coord.section_coord.z + 1)
    //         }
    //         super::visible::VisibleAxis::Positive => 0..max_graded_coord.section_coord.z,
    //         super::visible::VisibleAxis::Negative => {
    //             min_graded_coord.section_coord.z..TERRAIN_CHUNK_SECTION_SIZE
    //         }
    //         super::visible::VisibleAxis::Full => 0..TERRAIN_CHUNK_SECTION_SIZE,
    //     };
    //
    //     for x in xrange.clone() {
    //         for y in yrange.clone() {
    //             for z in zrange.clone() {
    //                 let coord = TerrainSectionCoord { x, y, z };
    //                 chunk_datas.push(VisibleVoxelSection {
    //                     data: match self.data.get(&coord) {
    //                         Some(data) => Some(data.clone()),
    //                         None => {
    //                             Some(Arc::new(TerrainSectionData::new(self.local_coord, coord)))
    //                         }
    //                     },
    //                     visible_direction: VisibleDirection {
    //                         x: match (
    //                             x == min_graded_coord.section_coord.x,
    //                             x == max_graded_coord.section_coord.x,
    //                         ) {
    //                             (true, true) => super::visible::VisibleAxis::Unqiue,
    //                             (true, false) => super::visible::VisibleAxis::Negative,
    //                             (false, true) => super::visible::VisibleAxis::Positive,
    //                             (false, false) => super::visible::VisibleAxis::Full,
    //                         },
    //                         y: match (
    //                             y == min_graded_coord.section_coord.y,
    //                             y == max_graded_coord.section_coord.y,
    //                         ) {
    //                             (true, true) => super::visible::VisibleAxis::Unqiue,
    //                             (true, false) => super::visible::VisibleAxis::Negative,
    //                             (false, true) => super::visible::VisibleAxis::Positive,
    //                             (false, false) => super::visible::VisibleAxis::Full,
    //                         },
    //                         z: match (
    //                             z == min_graded_coord.section_coord.z,
    //                             z == max_graded_coord.section_coord.z,
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
    //     chunk_datas
    // }
}
