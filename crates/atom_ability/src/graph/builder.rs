//! Effect Graph 构建器：定义图的名称并以编程方式构建图实体。

use std::fmt::Debug;

use bevy::prelude::{Commands, Entity, ResMut};

use super::context::InstantEffectNodeMap;

/// 标记 trait：所有子实体均为图节点的 Effect Graph 类型。
pub trait EffectGraph {}

/// Effect Graph 构建器：定义图的名称并以编程方式构建图实体。
pub trait EffectGraphBuilder: Debug + Sync + Send {
    // TODO: move to another trait
    /// 返回图名，用于查找对应的图实例。
    fn get_effect_graph_name(&self) -> &'static str;

    /// 在 `commands` 中构建图实体，并将即时创建的节点登记到 `instant_map`，返回图根实体。
    fn build(
        &self,
        commands: &mut Commands,
        instant_map: &mut ResMut<InstantEffectNodeMap>,
    ) -> Entity;
}
