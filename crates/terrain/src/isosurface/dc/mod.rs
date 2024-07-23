pub mod dc_gizmos;
pub mod dual_contouring;
pub mod main_mesh;
pub mod octree;
pub mod seam_mesh;

use std::sync::{Arc, RwLock};

use bevy::{prelude::*, utils::HashMap};
use dc_gizmos::DcGizmosPlugin;

use crate::{
    chunk_mgr::chunk::chunk_lod::OctreeDepthType, setting::TerrainSetting, TerrainSystemSet,
};

use octree::address::{construct_octree_depth_coord_map, NodeAddress};

use super::{
    comp::{
        read_chunk_update_lod_event, read_chunk_update_seam_event, TerrainChunkCreateMainMeshEvent,
        TerrainChunkCreateSeamMeshEvent,
    },
    mesh::{create_main_mesh, create_seam_mesh},
    IsosurfaceSystemSet,
};

#[derive(Debug, Default, Reflect)]
pub struct DualContouringPlugin;

impl Plugin for DualContouringPlugin {
    fn build(&self, app: &mut App) {
        let terrain_setting = app.world().get_resource::<TerrainSetting>().unwrap();
        app.insert_resource(OctreeDepthCoordMapper {
            mapper: Arc::new(RwLock::new(construct_octree_depth_coord_map(
                terrain_setting.chunk_setting.chunk_size * 4.0,
                terrain_setting.chunk_setting.get_voxel_size(0),
            ))),
        })
        .configure_sets(
            Update,
            (
                IsosurfaceSystemSet::GenerateMainMesh,
                IsosurfaceSystemSet::GenerateSeamMesh,
            )
                .chain()
                .in_set(TerrainSystemSet::GenerateTerrain),
        )
        .add_plugins(DcGizmosPlugin)
        .add_event::<TerrainChunkCreateMainMeshEvent>()
        .add_event::<TerrainChunkCreateSeamMeshEvent>()
        .add_systems(
            Update,
            (
                read_chunk_update_lod_event,
                main_mesh::construct_octree,
                main_mesh::simplify_octree,
                main_mesh::dual_contouring,
                create_main_mesh,
            )
                .chain()
                .in_set(IsosurfaceSystemSet::GenerateMainMesh),
        )
        .add_systems(
            Update,
            (
                read_chunk_update_seam_event,
                seam_mesh::construct_octree,
                // seam_mesh::simplify_octree,
                seam_mesh::dual_contouring,
                create_seam_mesh,
            )
                .chain()
                .in_set(IsosurfaceSystemSet::GenerateSeamMesh),
        );
    }
}

#[derive(Resource, Default, Debug)]
pub struct OctreeDepthCoordMapper {
    /// depth is from 1 to n
    mapper: Arc<RwLock<HashMap<OctreeDepthType, Vec<NodeAddress>>>>,
}
