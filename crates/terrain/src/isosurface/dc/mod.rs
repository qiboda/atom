pub mod dc_gizmos;
pub mod dual_contouring;
pub mod octree;

use std::{
    ops::Not,
    sync::{Arc, RwLock},
};

use bevy::{prelude::*, utils::HashMap};
use bevy_async_task::AsyncTaskPool;
use dc_gizmos::DcGizmosPlugin;
use dual_contouring::DefaultDualContouringVisiter;
use ndshape::{RuntimeShape, Shape};
use terrain_core::chunk::coords::TerrainChunkCoord;

use crate::{
    chunk_mgr::chunk::{bundle::TerrainChunk, chunk_lod::LodType},
    setting::TerrainSetting,
};

use octree::{
    address::{construct_octree_depth_coord_map, CellAddress, DepthCoordMap},
    cell::Cell,
    Octree, OctreeProxy,
};

use super::{
    comp::{
        read_chunk_udpate_lod_event, IsosurfaceState, TerrainChunkGenerator,
        TerrainChunkMainMeshCreatedEvent, TerrainChunkUpdateLodEvent,
    },
    mesh::{create_mesh, mesh_info::MeshInfo},
    surface::shape_surface::{IsosurfaceContext, ShapeSurface},
};

#[derive(Debug, Default, Reflect)]
pub struct DualContouringPlugin;

impl Plugin for DualContouringPlugin {
    fn build(&self, app: &mut App) {
        let terrain_setting = app.world().get_resource::<TerrainSetting>().unwrap();
        let size = (terrain_setting.chunk_settings.chunk_size
            / terrain_setting.chunk_settings.voxel_size) as u32;
        let shape = RuntimeShape::<u32, 3>::new([size, size, size]);

        app.insert_resource(OctreeDepthCoordMapper {
            mapper: Arc::new(RwLock::new(construct_octree_depth_coord_map(&shape))),
        })
        .add_plugins(DcGizmosPlugin)
        .add_event::<TerrainChunkUpdateLodEvent>()
        .add_event::<TerrainChunkMainMeshCreatedEvent>()
        .add_systems(PreUpdate, read_chunk_udpate_lod_event)
        .add_systems(
            Update,
            (
                construct_octree,
                simplity_octree,
                dual_contouring,
                create_mesh,
            )
                .chain(),
        );
    }
}

#[derive(Resource, Default, Debug)]
pub struct OctreeDepthCoordMapper {
    /// depth is from 1 to n
    mapper: Arc<RwLock<HashMap<u16, Vec<CellAddress>>>>,
}

#[allow(clippy::too_many_arguments)]
async fn construct_octree_task(
    entity: Entity,
    surface: Arc<RwLock<ShapeSurface>>,
    cell_address_mapper: Arc<RwLock<HashMap<u16, Vec<CellAddress>>>>,
    lod: LodType,
    chunk_size: f32,
    voxel_size: f32,
    chunk_coord: TerrainChunkCoord,
    std_dev_pos: f32,
    std_dev_normal: f32,
) -> (Entity, Octree) {
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
        cell_address_mapper,
    );

    Octree::check_children_relation(octree.address_cell_map.clone());

    (entity, octree)
}

#[allow(clippy::type_complexity)]
fn construct_octree(
    mut commands: Commands,
    mut task_pool: AsyncTaskPool<(Entity, Octree)>,
    chunk_query: Query<&TerrainChunkCoord, With<TerrainChunk>>,
    mut chunk_generator_query: ParamSet<(
        Query<(Entity, &Parent, &IsosurfaceState, &TerrainChunkGenerator)>,
        Query<&mut IsosurfaceState, With<TerrainChunkGenerator>>,
    )>,
    setting: Res<TerrainSetting>,
    surface: Res<IsosurfaceContext>,
    cell_mapper: Res<OctreeDepthCoordMapper>,
) {
    if task_pool.is_idle() {
        for (entity, parent, state, generator) in chunk_generator_query.p0().iter() {
            if state == &IsosurfaceState::ConstructOctree {
                debug!("construct_octree");
                if let Ok(chunk_coord) = chunk_query.get(parent.get()) {
                    let chunk_size = setting.chunk_settings.chunk_size;
                    let voxel_size = setting.chunk_settings.voxel_size;
                    let lod = generator.lod;
                    let shape_surface = surface.shape_surface.clone();
                    let mapper = cell_mapper.mapper.clone();
                    let stddev_pos = setting.chunk_settings.qef_pos_stddev;
                    let stddev_normal = setting.chunk_settings.qef_normal_stddev;
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
                        *state = IsosurfaceState::SimplifyOctree;
                    }
                }
            }
        }
    }
}

async fn simplity_octree_task(
    entity: Entity,
    deep_coord_mapper: Arc<RwLock<DepthCoordMap>>,
    address_cell_map: Arc<RwLock<HashMap<CellAddress, Cell>>>,
    cell_shape: RuntimeShape<u32, 3>,
    qef_threshold_map: HashMap<u16, f32>,
) -> Entity {
    Octree::simplify_octree(
        address_cell_map.clone(),
        cell_shape,
        deep_coord_mapper,
        qef_threshold_map,
    );

    Octree::check_children_relation(address_cell_map.clone());

    entity
}

#[allow(clippy::type_complexity)]
fn simplity_octree(
    mut task_pool: AsyncTaskPool<Entity>,
    mut chunk_generator_query: ParamSet<(
        Query<(Entity, &Octree, &IsosurfaceState), With<TerrainChunkGenerator>>,
        Query<&mut IsosurfaceState, With<TerrainChunkGenerator>>,
    )>,
    settings: Res<TerrainSetting>,
    depth_coord_mapper: Res<OctreeDepthCoordMapper>,
) {
    if settings.chunk_settings.qef_solver.not() {
        for mut state in chunk_generator_query.p1().iter_mut() {
            if *state == IsosurfaceState::SimplifyOctree {
                *state = IsosurfaceState::DualContouring;
            }
        }
        return;
    }

    if task_pool.is_idle() {
        for (entity, octree, state) in chunk_generator_query.p0().iter() {
            if state == &IsosurfaceState::SimplifyOctree {
                debug!("simplity_octree");
                let mapper = depth_coord_mapper.mapper.clone();
                let address_cell_map = octree.address_cell_map.clone();
                let cell_shape = octree.cell_shape.clone();
                let qef_threshold_map = settings.chunk_settings.qef_solver_threshold.clone();
                task_pool.spawn(simplity_octree_task(
                    entity,
                    mapper,
                    address_cell_map,
                    cell_shape,
                    qef_threshold_map,
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
                    *state = IsosurfaceState::DualContouring;
                }
            }
        }
    }
}

async fn dual_contouring_run_task(
    entity: Entity,
    surface: Arc<RwLock<ShapeSurface>>,
    cell_addresses: Arc<RwLock<HashMap<CellAddress, Cell>>>,
) -> (Entity, MeshInfo) {
    let surface: std::sync::RwLockReadGuard<ShapeSurface> = surface.read().unwrap();

    let mut mesh_info = MeshInfo::default();

    let mut default_visiter = DefaultDualContouringVisiter::new(&surface);
    let octree = OctreeProxy {
        cell_addresses: cell_addresses.read().unwrap(),
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
pub fn dual_contouring(
    mut commands: Commands,
    mut task_pool: AsyncTaskPool<(Entity, MeshInfo)>,
    mut chunk_generator_query: ParamSet<(
        Query<(Entity, &Octree, &IsosurfaceState), With<TerrainChunkGenerator>>,
        Query<&mut IsosurfaceState, With<TerrainChunkGenerator>>,
    )>,
    surface: Res<IsosurfaceContext>,
) {
    if task_pool.is_idle() {
        for (entity, octree, state) in chunk_generator_query.p0().iter() {
            if state == &IsosurfaceState::DualContouring {
                let shape_surface = surface.shape_surface.clone();
                let octree_cell_address = octree.address_cell_map.clone();
                debug!("dual_contouring");
                task_pool.spawn(dual_contouring_run_task(
                    entity,
                    shape_surface,
                    octree_cell_address,
                ));
            }
        }
    }

    for status in task_pool.iter_poll() {
        match status {
            bevy_async_task::AsyncTaskStatus::Idle => {}
            bevy_async_task::AsyncTaskStatus::Pending => {}
            bevy_async_task::AsyncTaskStatus::Finished((entity, mesh_info)) => {
                debug!("dual_contouring end");
                if let Some(mut entity_cmds) = commands.get_entity(entity) {
                    entity_cmds.insert(mesh_info);
                    if let Ok(mut state) = chunk_generator_query.p1().get_mut(entity) {
                        *state = IsosurfaceState::CreateMesh;
                        debug!("dual_contouring end over");
                    }
                }
            }
        }
    }
}
