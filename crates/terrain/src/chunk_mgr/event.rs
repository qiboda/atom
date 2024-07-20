use std::ops::Not;

use bevy::{prelude::*, utils::hashbrown::HashSet};
use ndshape::{AbstractShape, ConstShape, ConstShape3i64};
use terrain_core::chunk::coords::TerrainChunkCoord;

use crate::isosurface::comp::{
    MainMeshState, SeamMeshState, TerrainChunkCreateSeamMeshEvent, TerrainChunkMainGenerator,
    TerrainChunkSeamGenerator,
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
    mut query: Query<
        (
            &Children,
            &mut TerrainChunkState,
            &TerrainChunkLod,
            &TerrainChunkCoord,
        ),
        With<TerrainChunk>,
    >,
    mut generator_query: Query<(
        Option<&mut Visibility>,
        &MainMeshState,
        &TerrainChunkMainGenerator,
    )>,
) {
    for (children, mut chunk_state, chunk_lod, chunk_coord) in query.iter_mut() {
        if TerrainChunkState::CreateMainMesh == *chunk_state {
            let mut count = 0;
            for child in children.iter() {
                if let Ok((visibility, mesh_state, generator)) = generator_query.get_mut(*child) {
                    if *mesh_state == MainMeshState::Done && generator.lod == chunk_lod.get_lod() {
                        debug!(
                            "update_to_wait_create_seam:{}, lod: {}",
                            chunk_coord,
                            chunk_lod.get_lod()
                        );
                        *chunk_state = TerrainChunkState::WaitToCreateSeam;
                        if let Some(mut visibility) = visibility {
                            *visibility = Visibility::Visible;
                        }
                        count += 1;
                    }
                }
            }
            assert!(count < 2);
        }
    }
}

pub fn hidden_main_mesh(
    query: Query<
        (
            &Children,
            &TerrainChunkLod,
            &TerrainChunkState,
            &TerrainChunkCoord,
        ),
        With<TerrainChunk>,
    >,
    mut generator_query: Query<(&ViewVisibility, &mut Visibility, &TerrainChunkMainGenerator)>,
) {
    let all_create_over = query.iter().all(|(_, _, state, _)| {
        if *state == TerrainChunkState::Done {
            return true;
        }
        false
    });

    if all_create_over.not() {
        return;
    }

    for (children, chunk_lod, _, chunk_coord) in query.iter() {
        debug!("main_mesh_visibility: {}", chunk_coord);
        for child in children.iter() {
            if let Ok((_, mut visibility, generator)) = generator_query.get_mut(*child) {
                if generator.lod != chunk_lod.get_lod() {
                    *visibility = Visibility::Hidden;
                }
            }
        }
    }
}

// 检测是否所有的主mesh都创建了，如果都创建了，且有需要更新缝隙的，更新缝隙
#[allow(clippy::type_complexity)]
pub fn to_create_seam_mesh(
    mut query: ParamSet<(
        Query<&TerrainChunkState, With<TerrainChunk>>,
        Query<(&TerrainChunkCoord, &TerrainChunkState), With<TerrainChunk>>,
        Query<(&mut TerrainChunkState, &mut SeamMeshIdGenerator), With<TerrainChunk>>,
    )>,
    chunk_mapper: Res<TerrainChunkMapper>,
    mut event_writer: EventWriter<TerrainChunkCreateSeamMeshEvent>,
) {
    let exist_create_main_mesh_state = query.p0().iter().any(|state| {
        if *state == TerrainChunkState::CreateMainMesh {
            debug!("state is create main mesh, state: {:?}", state);
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
        if TerrainChunkState::WaitToCreateSeam == *state {
            to_create_seam_chunks.push(chunk_coords);
        }
    }

    let mut update_seam_chunk_coords = HashSet::new();
    to_create_seam_chunks.iter().for_each(|x| {
        for i in 0..ConstShape3i64::<2, 2, 2>::SIZE {
            let offset: TerrainChunkCoord = ConstShape3i64::<2, 2, 2>.delinearize(i).into();
            // 增加轴向负方向的8个chunk，其中一个是x。
            // 因为负方向的chunk和 x共享边界，所以需要更新。
            let new_chunk_coord = *x - &offset;
            update_seam_chunk_coords.insert(new_chunk_coord);
        }
    });

    for chunk_coord in update_seam_chunk_coords {
        if let Some(entity) = chunk_mapper.get_chunk_entity_by_coord(chunk_coord) {
            if let Ok((mut state, mut seam_mesh_id_generator)) = query.p2().get_mut(*entity) {
                let seam_mesh_id = seam_mesh_id_generator.gen();
                *state = TerrainChunkState::CreateSeamMesh;
                info!(
                    "to create seam chunks, coords: {}, id:{:?}, current id: {:?}",
                    chunk_coord,
                    seam_mesh_id,
                    seam_mesh_id_generator.current(),
                );
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
    mut generator_query: Query<(&SeamMeshState, &TerrainChunkSeamGenerator)>,
) {
    for (children, mut chunk_state, chunk_coord, id_generator) in query.iter_mut() {
        if TerrainChunkState::CreateSeamMesh == *chunk_state {
            let mut count = 0;
            for child in children {
                if let Ok((state, seam_generator)) = generator_query.get_mut(*child) {
                    if *state == SeamMeshState::Done {
                        count += 1;
                        if id_generator.current() == seam_generator.seam_mesh_id {
                            info!(
                                "update_create_seam_mesh_over: {}, {:?}",
                                chunk_coord,
                                id_generator.current()
                            );
                            *chunk_state = TerrainChunkState::Done;
                        }
                    }
                }
            }
            assert!(count < 2);
        }
    }
}
