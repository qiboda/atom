use bevy::{prelude::*, utils::hashbrown::HashSet};
use ndshape::{AbstractShape, ConstShape, ConstShape3i64};
use terrain_core::chunk::coords::TerrainChunkCoord;

use crate::isosurface::{
    comp::{
        MainMeshState, SeamMeshState, TerrainChunkCreateSeamMeshEvent, TerrainChunkMainGenerator,
        TerrainChunkSeamGenerator,
    },
    dc::seam_mesh,
};

use super::{
    chunk::{
        bundle::TerrainChunk,
        chunk_lod::TerrainChunkLod,
        state::{SeamMeshIdGenerator, TerrainChunkState},
    },
    chunk_mapper::TerrainChunkMapper,
};

pub fn update_to_wait_create_seam(
    mut query: Query<(&Children, &mut TerrainChunkState, &TerrainChunkCoord), With<TerrainChunk>>,
    mut generator_query: Query<(
        Option<&mut Visibility>,
        &MainMeshState,
        &TerrainChunkMainGenerator,
    )>,
) {
    for (children, mut chunk_state, chunk_coord) in query.iter_mut() {
        if let TerrainChunkState::CreateMainMesh(lod) = *chunk_state {
            let mut count = 0;
            for child in children.iter() {
                if let Ok((visibility, mesh_state, generator)) = generator_query.get_mut(*child) {
                    if *mesh_state == MainMeshState::Done && generator.lod == lod {
                        info!("update_to_wait_create_seam:{}, lod: {}", chunk_coord, lod);
                        *chunk_state = TerrainChunkState::WaitToCreateSeam(lod);
                        if let Some(mut visibility) = visibility {
                            *visibility = Visibility::Visible;
                        }
                        count += 1;
                    } else if let Some(mut visibility) = visibility {
                        *visibility = Visibility::Hidden;
                    }
                }
            }
            assert!(count < 2);
        }
    }
}

// 检测是否所有的主mesh都创建了，如果都创建了，且有需要更新缝隙的，更新缝隙
#[allow(clippy::type_complexity)]
pub fn to_create_seam_mesh(
    mut query: ParamSet<(
        Query<&TerrainChunkState, With<TerrainChunk>>,
        Query<(&TerrainChunkCoord, &TerrainChunkState), With<TerrainChunk>>,
        Query<
            (
                &mut TerrainChunkState,
                &TerrainChunkLod,
                &mut SeamMeshIdGenerator,
            ),
            With<TerrainChunk>,
        >,
    )>,
    chunk_mapper: Res<TerrainChunkMapper>,
    mut event_writer: EventWriter<TerrainChunkCreateSeamMeshEvent>,
) {
    let exist_create_main_mesh_state = query.p0().iter().any(|state| {
        if matches!(*state, TerrainChunkState::CreateMainMesh(_)) {
            info!("state is create main mesh, state: {:?}", state);
            return true;
        }
        false
    });

    if exist_create_main_mesh_state {
        return;
    }

    // 找到所有刚刚创建了MainMesh的chunk
    let mut to_create_seam_chunks = vec![];
    for (chunk_coords, state) in query.p1().iter_inner() {
        if let TerrainChunkState::WaitToCreateSeam(_) = *state {
            to_create_seam_chunks.push(chunk_coords);
        }
    }

    let mut udpate_seam_chunk_coords = HashSet::new();
    to_create_seam_chunks.iter().for_each(|x| {
        for i in 0..ConstShape3i64::<2, 2, 2>::SIZE {
            let offset: TerrainChunkCoord = ConstShape3i64::<2, 2, 2>.delinearize(i).into();
            // 增加轴向负方向的8个chunk，其中一个是x。
            // 因为负方向的chunk和 x共享边界，所以需要更新。
            let new_chunk_coord = *x - &offset;
            udpate_seam_chunk_coords.insert(new_chunk_coord);
        }
    });

    for chunk_coord in udpate_seam_chunk_coords {
        if let Some(entity) = chunk_mapper.get_chunk_entity_by_coord(chunk_coord) {
            if let Ok((mut state, lod, mut seam_mesh_id_generator)) = query.p2().get_mut(*entity) {
                let seam_mesh_id = seam_mesh_id_generator.next();
                *state = TerrainChunkState::CreateSeamMesh(seam_mesh_id);
                info!("to crate seam chunks coords: {}", chunk_coord);
                event_writer.send(TerrainChunkCreateSeamMeshEvent {
                    chunk_entity: *entity,
                    seam_mesh_id,
                });
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_create_seam_mesh_over(
    mut query: Query<
        (
            &Children,
            &mut TerrainChunkState,
            &TerrainChunkCoord,
            &SeamMeshIdGenerator,
        ),
        With<TerrainChunk>,
    >,
    mut generator_query: Query<(
        Option<&mut Visibility>,
        &mut SeamMeshState,
        &TerrainChunkSeamGenerator,
    )>,
) {
    for (children, mut chunk_state, chunk_coord, id_generator) in query.iter_mut() {
        if let TerrainChunkState::CreateSeamMesh(seam_mesh_id) = *chunk_state {
            let mut count = 0;
            info!(
                "update_create_seam_mesh_over: {}, seam mesh id:{:?}",
                chunk_coord, seam_mesh_id
            );
            for child in children {
                if let Ok((visibility, mut state, seam_generator)) = generator_query.get_mut(*child)
                {
                    if *state == SeamMeshState::Done {
                        count += 1;
                        if seam_mesh_id == seam_generator.seam_mesh_id {
                            info!("update_create_seam_mesh_over: {:?}", visibility);
                            *chunk_state = TerrainChunkState::Done;
                            if let Some(mut visibility) = visibility {
                                *visibility = Visibility::Visible;
                            }
                        } else {
                            *state = SeamMeshState::PendingRemove;
                        }
                    }
                }
            }
            assert!(count < 3);
        }
    }
}

pub fn remove_unused_seam_mesh(
    chunk_query: Query<(&Children, &TerrainChunkCoord, &SeamMeshIdGenerator), With<TerrainChunk>>,
    query: Query<(
        Entity,
        &Parent,
        &SeamMeshState,
        &ViewVisibility,
        &TerrainChunkSeamGenerator,
    )>,
    mut commands: Commands,
) {
    for (entity, parent, state, view_visibility, seam_generator) in query.iter() {
        if *state == SeamMeshState::PendingRemove {
            let mut can_destroy = false;
            if let Ok((children, coord, id_generator)) = chunk_query.get(parent.get()) {
                for child in children.iter() {
                    if let Ok((_, _, seam_state, view_visibility, seam_generator)) =
                        query.get(*child)
                    {
                        if seam_generator.seam_mesh_id == id_generator.current()
                            && view_visibility.get()
                        {
                            can_destroy = true;
                            break;
                        }
                    }
                }
            }

            if can_destroy {
                let (_, coord, _) = chunk_query.get(parent.get()).unwrap();
                info!("destroy seam mesh: {}", coord);
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
