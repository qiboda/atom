use std::ops::Not;

use bevy::prelude::*;

use crate::chunk_mgr::chunk::{bundle::TerrainChunk, chunk_lod::LodType};

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
pub enum IsosurfaceState {
    ConstructOctree,
    SimplifyOctree,
    DualContouring,
    // SimplifyMesh,
    CreateMesh,
    Done,
}

#[derive(Debug, Default, PartialEq, Eq, Copy, Clone, Component)]
pub struct TerrainChunkGenerator {
    pub lod: LodType,
}

#[derive(Debug, Event, Clone, Copy)]
pub struct TerrainChunkUpdateLodEvent {
    pub entity: Entity,
    pub lod: LodType,
}

#[derive(Debug, Event, Clone, Copy)]
pub struct TerrainChunkMainMeshCreatedEvent {
    pub chunk_entity: Entity,
    pub lod: LodType,
}

pub(crate) fn read_chunk_udpate_lod_event(
    mut event_reader: EventReader<TerrainChunkUpdateLodEvent>,
    chunk_children: Query<&Children, With<TerrainChunk>>,
    chunk_generator: Query<(&TerrainChunkGenerator, Option<&Handle<Mesh>>)>,
    mut commands: Commands,
    mut event_writer: EventWriter<TerrainChunkMainMeshCreatedEvent>,
) {
    for event in event_reader.read() {
        match chunk_children.get(event.entity) {
            Ok(chunk_children) => {
                let mut find_lod_generator = false;
                for child in chunk_children.iter() {
                    if let Ok((generator, mesh)) = chunk_generator.get(*child) {
                        if generator.lod == event.lod {
                            find_lod_generator = true;
                            if mesh.is_some() {
                                event_writer.send(TerrainChunkMainMeshCreatedEvent {
                                    chunk_entity: event.entity,
                                    lod: generator.lod,
                                });
                            }
                        }
                    }
                }

                if find_lod_generator.not() {
                    commands
                        .spawn((
                            TerrainChunkGenerator { lod: event.lod },
                            IsosurfaceState::ConstructOctree,
                        ))
                        .set_parent(event.entity);
                }
            }
            Err(_) => {
                commands
                    .spawn((
                        TerrainChunkGenerator { lod: event.lod },
                        IsosurfaceState::ConstructOctree,
                    ))
                    .set_parent(event.entity);
            }
        }
    }
}
