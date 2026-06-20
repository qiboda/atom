use std::collections::HashMap;

use bevy::{
    math::Vec3,
    prelude::*,
    render::extract_resource::ExtractResource,
};

/// 从主世界同步到渲染世界的待处理 chunk (entity → world_min 坐标)
#[derive(Resource, Clone, ExtractResource, Default)]
pub struct TerrainChunksToProcess {
    pub pending: HashMap<Entity, Vec3>,
}
