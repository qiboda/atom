pub mod dc_gizmos;
pub mod dual_contouring;
pub mod main_mesh;
pub mod octree;
pub mod seam_mesh;

use std::sync::{Arc, RwLock};

use atom_internal::app_state::AppState;
use bevy::{prelude::*, transform::commands, utils::HashMap};
use dc_gizmos::DcGizmosPlugin;

use crate::{
    chunk_mgr::chunk::chunk_lod::OctreeDepthType, setting::TerrainSetting, TerrainSystemSet,
};

use octree::address::{construct_octree_depth_coord_map, NodeAddress};

use super::{
    comp::{
        trigger_chunk_update_seam_event, trigger_create_main_mesh_event,
        TerrainChunkCreateMainMeshEvent, TerrainChunkCreateSeamMeshEvent,
    },
    mesh::{create_main_mesh, create_seam_mesh},
    IsosurfaceSystemSet,
};

#[derive(Debug, Default, Reflect)]
pub struct DualContouringPlugin;

impl Plugin for DualContouringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::AppRunning), dual_contouring_init)
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
            .observe(trigger_create_main_mesh_event)
            .add_event::<TerrainChunkCreateMainMeshEvent>()
            .add_event::<TerrainChunkCreateSeamMeshEvent>()
            .add_systems(
                Update,
                (
                    main_mesh::construct_octree,
                    main_mesh::simplify_octree,
                    main_mesh::dual_contouring,
                    create_main_mesh,
                )
                    .chain()
                    .in_set(IsosurfaceSystemSet::GenerateMainMesh),
            )
            .observe(trigger_chunk_update_seam_event)
            .add_systems(
                Update,
                (
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

pub fn dual_contouring_init(world: &mut World) {
    let span = info_span!("OctreeDepthCoordMapper").entered();

    // TODO 在只有缝隙需要的更低的层次下，可以只保存边缘的坐标，减少内存占用。
    let terrain_setting = world.get_resource::<TerrainSetting>().unwrap();
    let mapper = construct_octree_depth_coord_map(
        terrain_setting.chunk_setting.chunk_size,
        terrain_setting.chunk_setting.get_voxel_size(0),
    );

    world.insert_resource(OctreeDepthCoordMapper {
        mapper: Arc::new(RwLock::new(mapper)),
    });

    drop(span);
}
