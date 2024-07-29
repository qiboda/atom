// use std::ops::Not;

// use bevy::prelude::*;

// use crate::chunk_mgr::chunk::{bundle::TerrainChunk, chunk_lod::LodType, state::SeamMeshId};

// /// 状态转换
// ///
// /// 添加chunk       -> 创建octree -> 简化octree -> 生成mesh info -> 简化mesh -> 创建mesh
// /// 判断是否填充缝隙 -> 构建octree -> 简化octree -> 生成mesh info -> 简化mesh -> 创建mesh
// ///
// /// 更新lod -> 创建octree -> 简化octree -> 生成mesh info -> 简化mesh -> 创建mesh
// /// 判断是否填充缝隙 -> 构建octree -> 简化octree -> 生成mesh info -> 简化mesh -> 创建mesh
// ///
// /// 判断是否填充缝隙: 检测正方向8个Chunk是否存在，存在则收集所有的相连叶子节点,填充缝隙
// ///
// /// 在chunk中存储所有lod对应的octree。
// ///
// /// 当在创建主mesh的过程中，收到lod的信息，等待主mesh创建完成后，再创建lod对应的mesh。
// ///
// /// 添加chunk和更新lod的事件，整合为一个。事件处理在Chunk上处理。
// /// 当所有的Chunk都创建完毕了，再开始创建缝隙。检测是否有chunk更新了chunk，但是没有更新seam。
// #[derive(PartialEq, Eq, Debug, Clone, Copy, Hash, Component)]
// pub enum MainMeshState {
//     ConstructOctree,
//     SimplifyOctree,
//     DualContouring,
//     // SimplifyMesh,
//     CreateMesh,
//     Done,
// }

// #[derive(PartialEq, Eq, Debug, Clone, Copy, Hash, Component)]
// pub enum SeamMeshState {
//     ConstructOctree,
//     DualContouring,
//     // SimplifyMesh,
//     CreateMesh,
//     Done,
//     PendingRemove,
// }

// #[derive(Debug, Default, PartialEq, Eq, Copy, Clone, Component)]
// pub struct TerrainChunkMainGenerator {
//     pub lod: LodType,
// }

// #[derive(Debug, PartialEq, Eq, Clone, Component)]
// pub struct TerrainChunkSeamGenerator {
//     pub(crate) seam_mesh_id: SeamMeshId,
// }

// #[derive(Debug, Event, Clone, Copy)]
// pub struct TerrainChunkCreateMainMeshEvent {
//     pub entity: Entity,
//     pub lod: LodType,
// }

// #[derive(Debug, Event, Clone, Copy)]
// pub struct TerrainChunkCreateSeamMeshEvent {
//     pub chunk_entity: Entity,
//     pub seam_mesh_id: SeamMeshId,
// }

// pub(crate) fn trigger_create_main_mesh_event(
//     event_trigger: Trigger<TerrainChunkCreateMainMeshEvent>,
//     chunk_children: Query<&Children, With<TerrainChunk>>,
//     mut chunk_generator: Query<(&TerrainChunkMainGenerator, &mut MainMeshState)>,
//     mut commands: Commands,
// ) {
//     let event = event_trigger.event();
//     let mut find_lod_generator = false;
//     if let Ok(chunk_children) = chunk_children.get(event.entity) {
//         for child in chunk_children.iter() {
//             if let Ok((generator, mut state)) = chunk_generator.get_mut(*child) {
//                 if generator.lod == event.lod {
//                     *state = MainMeshState::ConstructOctree;
//                     find_lod_generator = true;
//                 }
//             }
//         }
//     }
//     if find_lod_generator.not() {
//         debug!(": {:?}", event.lod);
//         commands
//             .spawn((
//                 TerrainChunkMainGenerator { lod: event.lod },
//                 MainMeshState::ConstructOctree,
//                 Name::new("terrain chunk main mesh"),
//             ))
//             .set_parent(event.entity);
//     }
// }

// /// 获取周围的chunk，如果有两个或者以上的chunk，则创建缝隙，lod选择尽可能小的。
// pub(crate) fn trigger_chunk_update_seam_event(
//     event_trigger: Trigger<TerrainChunkCreateSeamMeshEvent>,
//     chunk_query: Query<&Children, With<TerrainChunk>>,
//     mut seam_query: Query<(&mut TerrainChunkSeamGenerator, &mut SeamMeshState)>,
//     mut commands: Commands,
// ) {
//     let event = event_trigger.event();
//     if let Ok(children) = chunk_query.get(event.chunk_entity) {
//         let mut find_seam_generator = false;
//         for child in children {
//             if let Ok((mut generator, mut state)) = seam_query.get_mut(*child) {
//                 generator.seam_mesh_id = event.seam_mesh_id;
//                 *state = SeamMeshState::ConstructOctree;
//                 find_seam_generator = true;
//             }
//         }
//         if find_seam_generator.not() {
//             commands
//                 .spawn((
//                     TerrainChunkSeamGenerator {
//                         seam_mesh_id: event.seam_mesh_id,
//                     },
//                     SeamMeshState::ConstructOctree,
//                     Name::new("terrain chunk seam mesh"),
//                 ))
//                 .set_parent(event.chunk_entity);
//         }
//     }
// }
