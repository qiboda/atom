/// 不支持lod，但是性能足够好，可以支持128尺寸的地形。
/// fixed mesh delete too early.需要延迟到新的mesh创建后再删除。
use std::sync::{Arc, RwLock};

use bevy::prelude::*;
use bevy_async_task::AsyncTaskPool;
use fast_surface_nets::{
    ndshape::{RuntimeShape, Shape},
    surface_nets, SurfaceNetsBuffer,
};
use terrain_core::chunk::coords::TerrainChunkCoord;

use crate::{
    chunk::{chunk_data::TerrainChunkData, TerrainChunk},
    setting::TerrainSetting,
};

use super::{
    lod::update_mesh_lod,
    mesh::{create_mesh, mesh_info::MeshInfo},
    state::IsosurfaceState,
    surface::shape_surface::{IsosurfaceContext, ShapeSurface},
};

#[derive(Debug, Default)]
pub struct SurfaceNetsPlugin;

impl Plugin for SurfaceNetsPlugin {
    fn build(&self, app: &mut App) {
        app.observe(trigger_on_add_terrain_chunk).add_systems(
            Update,
            (gen_mesh_info, gen_mesh_info, create_mesh, update_mesh_lod).chain(),
        );
    }
}

fn trigger_on_add_terrain_chunk(
    trigger: Trigger<OnAdd, TerrainChunk>,
    mut commands: Commands,
    query: Query<(), (With<TerrainChunk>, Without<IsosurfaceState>)>,
) {
    let entity = trigger.entity();
    if let Ok(()) = query.get(entity) {
        commands.entity(entity).insert(IsosurfaceState::GenMeshInfo);
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

async fn surface_nets_run_task(
    entity: Entity,
    surface: Arc<RwLock<ShapeSurface>>,
    chunk_size: f32,
    chunk_coord: TerrainChunkCoord,
    lod: u8,
) -> (Entity, MeshInfo) {
    let offset = chunk_coord * chunk_size;
    let grid_size = 1.0_f32.powi(lod as i32);
    let sampler_size = ((chunk_size) / grid_size) as u32 + 2;
    let shape = RuntimeShape::<u32, 3>::new([sampler_size, sampler_size, sampler_size]);

    let mut samples = Vec::with_capacity(shape.size() as usize);
    let surface: std::sync::RwLockReadGuard<ShapeSurface> = surface.read().unwrap();

    for i in 0..shape.size() {
        let loc = offset + Vec3::from_array(shape.delinearize(i).map(|v| v as f32)) * grid_size;
        let density = surface.get_value_from_vec(loc);
        samples.push(density);
    }

    info!(
        "surface_nets_run_task: samples: {:?}, sampler size: {:?}",
        samples.len(),
        sampler_size
    );

    let mut buffer = SurfaceNetsBuffer::default();
    surface_nets(&samples, &shape, [0; 3], [sampler_size - 1; 3], &mut buffer);

    let mut mesh_info = MeshInfo::default();
    mesh_info.set_vertice_positions(
        buffer
            .positions
            .into_iter()
            .map(|v| Vec3::new(v[0], v[1], v[2]) * grid_size + offset)
            .collect(),
    );
    mesh_info.set_vertice_normals(buffer.normals.into_iter().map(|v| v.into()).collect());
    mesh_info.set_indices(buffer.indices);
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
                &TerrainChunkCoord,
                &TerrainChunkData,
                &IsosurfaceState,
            ),
            With<TerrainChunk>,
        >,
        Query<&mut IsosurfaceState, With<TerrainChunk>>,
    )>,
    settings: Res<TerrainSetting>,
    surface: Res<IsosurfaceContext>,
) {
    if task_pool.is_idle() {
        for (entity, chunk_coord, chuk_data, state) in chunk_query.p0().iter() {
            if state == &IsosurfaceState::GenMeshInfo {
                let lod = chuk_data.lod;
                let chunk_size = settings.chunk_settings.chunk_size;
                let shape_surface = surface.shape_surface.clone();
                task_pool.spawn(surface_nets_run_task(
                    entity,
                    shape_surface,
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
