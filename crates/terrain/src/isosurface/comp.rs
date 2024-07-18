use std::ops::Not;

use bevy::prelude::*;

use crate::chunk_mgr::chunk::{bundle::TerrainChunk, chunk_lod::LodType, state::SeamMeshId};

/// 状态转换
///
/// 添加chunk       -> 创建octree -> 简化octree -> 生成mesh info -> 简化mesh -> 创建mesh
/// 判断是否填充缝隙 -> 构建octree -> 简化octree -> 生成mesh info -> 简化mesh -> 创建mesh
///
/// 更新lod -> 创建octree -> 简化octree -> 生成mesh info -> 简化mesh -> 创建mesh
/// 判断是否填充缝隙 -> 构建octree -> 简化octree -> 生成mesh info -> 简化mesh -> 创建mesh
///
/// 判断是否填充缝隙: 检测正方向8个Chunk是否存在，存在则收集所有的相连叶子节点,填充缝隙
///
/// 在chunk中存储所有lod对应的octree。
///
/// 当在创建主mesh的过程中，收到lod的信息，等待主mesh创建完成后，再创建lod对应的mesh。
///
/// 添加chunk和更新lod的事件，整合为一个。事件处理在Chunk上处理。
/// 当所有的Chunk都创建完毕了，再开始创建缝隙。检测是否有chunk更新了chunk，但是没有更新seam。
#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash, Component)]
pub enum MainMeshState {
    ConstructOctree,
    SimplifyOctree,
    DualContouring,
    // SimplifyMesh,
    CreateMesh,
    Done,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash, Component)]
pub enum SeamMeshState {
    ConstructOctree,
    DualContouring,
    // SimplifyMesh,
    CreateMesh,
    Done,
    PendingRemove,
}

#[derive(Debug, Default, PartialEq, Eq, Copy, Clone, Component)]
pub struct TerrainChunkMainGenerator {
    pub lod: LodType,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Component)]
pub struct TerrainChunkSeamGenerator {
    pub(crate) seam_mesh_id: SeamMeshId,
}

#[derive(Debug, Event, Clone, Copy)]
pub struct TerrainChunkCreateMainMeshEvent {
    pub entity: Entity,
    pub lod: LodType,
}

#[derive(Debug, Event, Clone, Copy)]
pub struct TerrainChunkCreateSeamMeshEvent {
    pub chunk_entity: Entity,
    pub seam_mesh_id: SeamMeshId,
}

pub(crate) fn read_chunk_udpate_lod_event(
    mut event_reader: EventReader<TerrainChunkCreateMainMeshEvent>,
    chunk_children: Query<&Children, With<TerrainChunk>>,
    chunk_generator: Query<&TerrainChunkMainGenerator>,
    mut commands: Commands,
) {
    for event in event_reader.read() {
        let mut find_lod_generator = false;
        if let Ok(chunk_children) = chunk_children.get(event.entity) {
            for child in chunk_children.iter() {
                if let Ok(generator) = chunk_generator.get(*child) {
                    if generator.lod == event.lod {
                        find_lod_generator = true;
                    }
                }
            }
        }
        if find_lod_generator.not() {
            commands
                .spawn((
                    TerrainChunkMainGenerator { lod: event.lod },
                    MainMeshState::ConstructOctree,
                ))
                .set_parent(event.entity);
        }
    }
}

/// 获取周围的chunk，如果有两个或者以上的chunk，则创建缝隙，lod选择尽可能小的。
pub(crate) fn read_chunk_udpate_seam_event(
    mut event_reader: EventReader<TerrainChunkCreateSeamMeshEvent>,
    mut commands: Commands,
) {
    for event in event_reader.read() {
        commands
            .spawn((
                TerrainChunkSeamGenerator {
                    seam_mesh_id: event.seam_mesh_id,
                },
                SeamMeshState::ConstructOctree,
            ))
            .set_parent(event.chunk_entity);
    }
}
