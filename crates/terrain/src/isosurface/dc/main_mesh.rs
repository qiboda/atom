use std::{
    ops::Not,
    sync::{Arc, RwLock},
};

use bevy::{prelude::*, utils::HashMap};
use ndshape::{RuntimeShape, Shape};

use crate::{
    chunk_mgr::chunk::{bundle::TerrainChunk, chunk_lod::LodType},
    isosurface::{
        comp::{MainMeshState, TerrainChunkMainGenerator},
        dc::octree::check_octree_nodes_relation,
        mesh::mesh_info::MeshInfo,
        surface::shape_surface::{IsosurfaceContext, ShapeSurface},
    },
    setting::TerrainSetting,
};

use super::{
    dual_contouring::{self, DefaultDualContouringVisiter},
    octree::{
        address::{NodeAddress, DepthCoordMap},
        node::Node,
        Octree, OctreeProxy,
    },
    OctreeDepthCoordMapper,
};
use bevy_async_task::AsyncTaskPool;
use terrain_core::chunk::coords::TerrainChunkCoord;

#[allow(clippy::too_many_arguments)]
async fn construct_octree_task(
    entity: Entity,
    surface: Arc<RwLock<ShapeSurface>>,
    node_address_mapper: Arc<RwLock<HashMap<u16, Vec<NodeAddress>>>>,
    lod: LodType,
    chunk_size: f32,
    voxel_size: f32,
    chunk_coord: TerrainChunkCoord,
    std_dev_pos: f32,
    std_dev_normal: f32,
) -> (Entity, Octree) {
    let _span = debug_span!("main mesh construct octree", %chunk_coord, lod).entered();

    let lod_voxel_size = voxel_size * 2.0_f32.powf((lod + 1) as f32);
    let offset = chunk_coord * chunk_size;
    let size = (chunk_size / lod_voxel_size) as u32 + 1;
    let shape = RuntimeShape::<u32, 3>::new([size, size, size]);
    debug!("lod_voxle size: {}, size: {}", lod_voxel_size, size);

    let mut samples = Vec::with_capacity(shape.usize());
    let surface: std::sync::RwLockReadGuard<ShapeSurface> = surface.read().unwrap();

    for i in 0..shape.size() {
        let loc =
            offset + Vec3::from_array(shape.delinearize(i).map(|v| v as f32)) * lod_voxel_size;
        let density = surface.get_value_from_vec(loc);
        samples.push(density);
    }

    let mut octree = Octree::new(RuntimeShape::<u32, 3>::new([size - 1, size - 1, size - 1]));
    Octree::build_bottom_up(
        &mut octree,
        &samples,
        &shape,
        lod_voxel_size,
        std_dev_pos,
        std_dev_normal,
        offset,
        &surface,
        node_address_mapper,
    );

    check_octree_nodes_relation!(octree.address_node_map.clone());

    (entity, octree)
}

#[allow(clippy::type_complexity)]
pub(crate) fn construct_octree(
    mut commands: Commands,
    mut task_pool: AsyncTaskPool<(Entity, Octree)>,
    chunk_query: Query<&TerrainChunkCoord, With<TerrainChunk>>,
    mut chunk_generator_query: ParamSet<(
        Query<(Entity, &Parent, &MainMeshState, &TerrainChunkMainGenerator)>,
        Query<&mut MainMeshState, With<TerrainChunkMainGenerator>>,
    )>,
    setting: Res<TerrainSetting>,
    surface: Res<IsosurfaceContext>,
    node_mapper: Res<OctreeDepthCoordMapper>,
) {
    if task_pool.is_idle() {
        for (entity, parent, state, generator) in chunk_generator_query.p0().iter() {
            if state == &MainMeshState::ConstructOctree {
                if let Ok(chunk_coord) = chunk_query.get(parent.get()) {
                    let lod = generator.lod;
                    let chunk_size = setting.chunk_setting.chunk_size;
                    let voxel_size = setting.chunk_setting.voxel_size;
                    let shape_surface = surface.shape_surface.clone();
                    let mapper = node_mapper.mapper.clone();
                    let stddev_pos = setting.chunk_setting.qef_pos_stddev;
                    let stddev_normal = setting.chunk_setting.qef_normal_stddev;

                    task_pool.spawn(construct_octree_task(
                        entity,
                        shape_surface,
                        mapper.clone(),
                        lod,
                        chunk_size,
                        voxel_size,
                        *chunk_coord,
                        stddev_pos,
                        stddev_normal,
                    ));
                }
            }
        }
    }

    for status in task_pool.iter_poll() {
        match status {
            bevy_async_task::AsyncTaskStatus::Idle => {}
            bevy_async_task::AsyncTaskStatus::Pending => {}
            bevy_async_task::AsyncTaskStatus::Finished((entity, octree)) => {
                if let Some(mut entity_cmds) = commands.get_entity(entity) {
                    entity_cmds.insert(octree);
                    if let Ok(mut state) = chunk_generator_query.p1().get_mut(entity) {
                        *state = MainMeshState::SimplifyOctree;
                    }
                }
            }
        }
    }
}

async fn simplity_octree_task(
    entity: Entity,
    deep_coord_mapper: Arc<RwLock<DepthCoordMap>>,
    address_node_map: Arc<RwLock<HashMap<NodeAddress, Node>>>,
    node_shape: RuntimeShape<u32, 3>,
    qef_threshold_map: HashMap<u16, f32>,
    chunk_coord: TerrainChunkCoord,
    lod: LodType,
) -> Entity {
    let _span = debug_span!("main mesh simplity octree task", %chunk_coord, lod).entered();

    Octree::simplify_octree(
        address_node_map.clone(),
        node_shape,
        deep_coord_mapper,
        qef_threshold_map,
    );

    check_octree_nodes_relation!(address_node_map.clone());

    entity
}

#[allow(clippy::type_complexity)]
pub(crate) fn simplity_octree(
    mut task_pool: AsyncTaskPool<Entity>,
    chunk_query: Query<&TerrainChunkCoord, With<TerrainChunk>>,
    mut chunk_generator_query: ParamSet<(
        Query<(
            Entity,
            &Parent,
            &Octree,
            &MainMeshState,
            &TerrainChunkMainGenerator,
        )>,
        Query<&mut MainMeshState, With<TerrainChunkMainGenerator>>,
    )>,
    settings: Res<TerrainSetting>,
    depth_coord_mapper: Res<OctreeDepthCoordMapper>,
) {
    if settings.chunk_setting.qef_solver.not() {
        for mut state in chunk_generator_query.p1().iter_mut() {
            if *state == MainMeshState::SimplifyOctree {
                *state = MainMeshState::DualContouring;
            }
        }
        return;
    }

    if task_pool.is_idle() {
        for (entity, parent, octree, state, generator) in chunk_generator_query.p0().iter() {
            if state == &MainMeshState::SimplifyOctree {
                let mapper = depth_coord_mapper.mapper.clone();
                let address_node_map = octree.address_node_map.clone();
                let node_shape = octree.node_shape.clone();
                let qef_threshold_map = settings.chunk_setting.qef_solver_threshold.clone();
                let chunk_coord = chunk_query.get(parent.get()).unwrap();
                let lod = generator.lod;
                task_pool.spawn(simplity_octree_task(
                    entity,
                    mapper,
                    address_node_map,
                    node_shape,
                    qef_threshold_map,
                    *chunk_coord,
                    lod,
                ));
            }
        }
    }

    for status in task_pool.iter_poll() {
        match status {
            bevy_async_task::AsyncTaskStatus::Idle => {}
            bevy_async_task::AsyncTaskStatus::Pending => {}
            bevy_async_task::AsyncTaskStatus::Finished(entity) => {
                if let Ok(mut state) = chunk_generator_query.p1().get_mut(entity) {
                    *state = MainMeshState::DualContouring;
                }
            }
        }
    }
}

async fn dual_contouring_run_task(
    entity: Entity,
    surface: Arc<RwLock<ShapeSurface>>,
    node_addresses: Arc<RwLock<HashMap<NodeAddress, Node>>>,
    chunk_coord: TerrainChunkCoord,
    lod: LodType,
) -> (Entity, MeshInfo) {
    let _span = debug_span!("main mesh dual contouring", %chunk_coord, lod).entered();

    let surface: std::sync::RwLockReadGuard<ShapeSurface> = surface.read().unwrap();

    let mut mesh_info = MeshInfo::default();

    let mut default_visiter = DefaultDualContouringVisiter::new(&surface);
    let octree = OctreeProxy {
        node_addresses: node_addresses.read().unwrap(),
        is_seam: false,
    };
    dual_contouring::dual_contouring(&octree, &mut default_visiter);

    mesh_info.positions = default_visiter.positions;
    mesh_info.normals = default_visiter
        .normals
        .iter()
        .map(|n| (*n).into())
        .collect();
    mesh_info.indices = default_visiter.tri_indices;
    (entity, mesh_info)
}

#[allow(clippy::type_complexity)]
pub(crate) fn dual_contouring(
    mut commands: Commands,
    mut task_pool: AsyncTaskPool<(Entity, MeshInfo)>,
    chunk_query: Query<&TerrainChunkCoord, With<TerrainChunk>>,
    mut chunk_generator_query: ParamSet<(
        Query<(
            Entity,
            &Parent,
            &Octree,
            &MainMeshState,
            &TerrainChunkMainGenerator,
        )>,
        Query<&mut MainMeshState, With<TerrainChunkMainGenerator>>,
    )>,
    surface: Res<IsosurfaceContext>,
) {
    if task_pool.is_idle() {
        for (entity, parent, octree, state, generator) in chunk_generator_query.p0().iter() {
            if state == &MainMeshState::DualContouring {
                let chunk_coord = chunk_query.get(parent.get()).unwrap();
                let lod = generator.lod;

                let shape_surface = surface.shape_surface.clone();
                let octree_node_address = octree.address_node_map.clone();
                task_pool.spawn(dual_contouring_run_task(
                    entity,
                    shape_surface,
                    octree_node_address,
                    *chunk_coord,
                    lod,
                ));
            }
        }
    }

    for status in task_pool.iter_poll() {
        match status {
            bevy_async_task::AsyncTaskStatus::Idle => {}
            bevy_async_task::AsyncTaskStatus::Pending => {}
            bevy_async_task::AsyncTaskStatus::Finished((entity, mesh_info)) => {
                if let Some(mut entity_cmds) = commands.get_entity(entity) {
                    entity_cmds.insert(mesh_info);
                    if let Ok(mut state) = chunk_generator_query.p1().get_mut(entity) {
                        *state = MainMeshState::CreateMesh;
                    }
                }
            }
        }
    }
}
