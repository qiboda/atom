pub mod dual_contouring;
pub mod octree;

use std::{
    ops::Not,
    sync::{Arc, RwLock},
};

use bevy::{
    color::palettes::css::{self, GREEN, LINEN, RED},
    math::bounding::BoundingVolume,
    prelude::*,
    utils::HashMap,
};
use bevy_async_task::AsyncTaskPool;
use dual_contouring::DefaultDualContouringVisiter;
use ndshape::{RuntimeShape, Shape};
use terrain_core::chunk::coords::TerrainChunkCoord;

use crate::{
    chunk::{chunk_data::TerrainChunkData, TerrainChunk},
    setting::TerrainSetting,
};

use octree::{
    address::{construct_octree_address_map, CellAddress},
    build_bottom_up,
    cell::{Cell, CellType},
    Octree, OctreeProxy,
};

use super::{
    mesh::{create_mesh, mesh_info::MeshInfo},
    state::IsosurfaceState,
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

        app.insert_resource(CellAddressMapper {
            mapper: Arc::new(RwLock::new(construct_octree_address_map(&shape))),
        })
        .init_gizmo_group::<OctreeCellGizmos>()
        .observe(trigger_on_add_terrain_chunk)
        .add_systems(
            Update,
            (
                debug_draw_octree_cell,
                construct_octree,
                gen_mesh_info,
                create_mesh,
            )
                .chain(),
        );
    }
}

#[derive(Resource, Default, Debug)]
pub struct CellAddressMapper {
    /// depth is from 1 to n
    mapper: Arc<RwLock<HashMap<u16, Vec<CellAddress>>>>,
}

// We can create our own gizmo config group!
#[derive(Default, Reflect, GizmoConfigGroup)]
struct OctreeCellGizmos {}

#[derive(Debug, Default, Component, Reflect, PartialEq, Eq)]
pub enum DualContouringState {
    #[default]
    ConstructOctree,
    DualContouring,
}

fn debug_draw_octree_cell(
    query: Query<&Octree>,
    mut octree_cell_gizmos: Gizmos<OctreeCellGizmos>,
) {
    return ;

    octree_cell_gizmos.axes(
        Transform {
            translation: Vec3::ZERO,
            rotation: Quat::from_axis_angle(Vec3::X, 0.0),
            scale: Vec3::ONE,
        },
        3.0,
    );

    for octree in query.iter() {
        let cell_addresses = octree.cell_addresses.read().unwrap();
        if cell_addresses.is_empty().not() {
            debug!("cell num: {}", cell_addresses.len());
            for (address, cell) in cell_addresses.iter() {
                if cell.cell_type == CellType::Leaf {
                    let loc = cell.vertex_estimate;
                    let normal = cell.normal_estimate;

                    octree_cell_gizmos.arrow(loc, loc + Vec3::from(normal) * 2.0, css::RED);

                    // info!("pos: {}", cell.vertex_estimate);
                    //     octree_cell_gizmos.cuboid(
                    //         Transform {
                    //             translation: cell.aabb.center().into(),
                    //             rotation: Quat::IDENTITY,
                    //             scale: (cell.aabb.half_size() * 2.0).into(),
                    //         },
                    //         css::RED,
                    //         // match address.depth() {
                    //         //     1 => css::RED,
                    //         //     2 => css::GREEN,
                    //         //     3 => css::BLUE,
                    //         //     4 => css::YELLOW,
                    //         //     5 => css::ORANGE,
                    //         //     6 => css::MAGENTA,
                    //         //     7 => css::WHITE,
                    //         //     _ => css::BLACK,
                    //         // },
                    //     );
                }
            }
        }
    }
}

fn trigger_on_add_terrain_chunk(
    trigger: Trigger<OnAdd, TerrainChunk>,
    mut commands: Commands,
    query: Query<(), (With<TerrainChunk>, Without<IsosurfaceState>)>,
) {
    let entity = trigger.entity();
    if let Ok(()) = query.get(entity) {
        if let Some(mut entity_cmds) = commands.get_entity(entity) {
            entity_cmds.insert((
                DualContouringState::ConstructOctree,
                IsosurfaceState::GenMeshInfo,
            ));
        }
    } else {
        warn!(
            "trigger_on_add_terrain_chunk: entity not found: {:?}",
            entity
        );
        panic!(
            "trigger_on_add_terrain_chunk: entity not found: {:?}",
            entity
        )
    }
}

async fn construct_octree_task(
    entity: Entity,
    surface: Arc<RwLock<ShapeSurface>>,
    cell_address_mapper: Arc<RwLock<HashMap<u16, Vec<CellAddress>>>>,
    chunk_size: f32,
    voxel_size: f32,
    chunk_coord: TerrainChunkCoord,
) -> (Entity, Octree) {
    let mut octree = Octree::default();

    let offset = chunk_coord * chunk_size;
    let size = (chunk_size / voxel_size) as u32 + 1;
    let shape = RuntimeShape::<u32, 3>::new([size, size, size]);

    let mut samples = Vec::with_capacity(shape.usize());
    let surface: std::sync::RwLockReadGuard<ShapeSurface> = surface.read().unwrap();

    for i in 0..shape.size() {
        let loc = offset + Vec3::from_array(shape.delinearize(i).map(|v| v as f32)) * voxel_size;
        let density = surface.get_value_from_vec(loc);
        samples.push(density);
    }

    // info!("samples: {:?}", samples);
    build_bottom_up(
        &mut octree,
        &samples,
        &shape,
        voxel_size,
        offset,
        &surface,
        cell_address_mapper,
    );

    // check octree children relation
    {
        let cell_addresses = octree.cell_addresses.read().unwrap();
        cell_addresses
            .iter()
            .for_each(|(address, cell)| match cell.cell_type {
                CellType::Branch => {
                    assert_eq!(cell.address, *address);
                    let mut exist_child = false;
                    for child_address in cell.address.get_children_addresses() {
                        if cell_addresses.get(&child_address).is_some() {
                            exist_child = true;
                            break;
                        }
                    }
                    assert!(exist_child);
                }
                CellType::Leaf => {
                    assert_eq!(cell.address, *address);
                    for child_address in cell.address.get_children_addresses() {
                        assert!(cell_addresses.get(&child_address).is_none());
                    }
                }
            });
    }

    (entity, octree)
}

#[allow(clippy::type_complexity)]
fn construct_octree(
    mut commands: Commands,
    mut task_pool: AsyncTaskPool<(Entity, Octree)>,
    mut chunk_query: ParamSet<(
        Query<
            (
                Entity,
                &TerrainChunkCoord,
                &IsosurfaceState,
                &DualContouringState,
            ),
            With<TerrainChunk>,
        >,
        Query<&mut DualContouringState, With<TerrainChunk>>,
    )>,
    settings: Res<TerrainSetting>,
    surface: Res<IsosurfaceContext>,
    cell_mapper: Res<CellAddressMapper>,
) {
    if task_pool.is_idle() {
        for (entity, chunk_coord, state, dc_state) in chunk_query.p0().iter() {
            if state == &IsosurfaceState::GenMeshInfo
                && dc_state == &DualContouringState::ConstructOctree
            {
                let chunk_size = settings.chunk_settings.chunk_size;
                let voxel_size = settings.chunk_settings.voxel_size;
                let shape_surface = surface.shape_surface.clone();
                let mapper = cell_mapper.mapper.clone();
                task_pool.spawn(construct_octree_task(
                    entity,
                    shape_surface,
                    mapper.clone(),
                    chunk_size,
                    voxel_size,
                    *chunk_coord,
                ));
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
                    if let Ok(mut state) = chunk_query.p1().get_mut(entity) {
                        *state = DualContouringState::DualContouring;
                    }
                }
            }
        }
    }
}

async fn dual_contouring_run_task(
    entity: Entity,
    surface: Arc<RwLock<ShapeSurface>>,
    cell_addresses: Arc<RwLock<HashMap<CellAddress, Cell>>>,
    chunk_size: f32,
    chunk_coord: TerrainChunkCoord,
    lod: u8,
) -> (Entity, MeshInfo) {
    let offset = chunk_coord * chunk_size;
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
    mesh_info.lod = lod;
    (entity, mesh_info)
}

#[allow(clippy::type_complexity)]
pub fn gen_mesh_info(
    mut commands: Commands,
    mut task_pool: AsyncTaskPool<(Entity, MeshInfo)>,
    mut chunk_query: ParamSet<(
        Query<
            (
                Entity,
                &Octree,
                &TerrainChunkCoord,
                &TerrainChunkData,
                &IsosurfaceState,
                &DualContouringState,
            ),
            With<TerrainChunk>,
        >,
        Query<&mut IsosurfaceState, With<TerrainChunk>>,
    )>,
    settings: Res<TerrainSetting>,
    surface: Res<IsosurfaceContext>,
) {
    if task_pool.is_idle() {
        for (entity, octree, chunk_coord, chuk_data, state, dc_state) in chunk_query.p0().iter() {
            if state == &IsosurfaceState::GenMeshInfo
                && dc_state == &DualContouringState::DualContouring
            {
                let lod = chuk_data.lod;
                let chunk_size = settings.chunk_settings.chunk_size;
                let shape_surface = surface.shape_surface.clone();
                let octree_cell_address = octree.cell_addresses.clone();
                task_pool.spawn(dual_contouring_run_task(
                    entity,
                    shape_surface,
                    octree_cell_address,
                    chunk_size,
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
                    if let Ok(mut state) = chunk_query.p1().get_mut(entity) {
                        *state = IsosurfaceState::CreateMesh;
                    }
                }
            }
        }
    }
}
