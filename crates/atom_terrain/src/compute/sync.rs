//! 主世界 → 渲染世界的 chunk 处理请求同步。
//! TerrainChunksToProcess 作为 ExtractResource 每帧同步待处理 chunk 列表。

use std::collections::HashMap;

use bevy::{
    math::Vec3,
    prelude::*,
    render::extract_resource::ExtractResource,
};

/// 从主世界同步到渲染世界的待处理 chunk (entity → world_min 坐标)
#[derive(Resource, Clone, ExtractResource, Default)]
pub struct TerrainChunksToProcess {
    /// 待处理 chunk: entity → world_min 坐标
    pub pending: HashMap<Entity, Vec3>,
}
