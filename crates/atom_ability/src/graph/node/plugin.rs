//! 节点基础设施插件：注册节点相关类型与事件过滤资源。

use std::any::TypeId;

use bevy::{app::Plugin, ecs::component::ComponentId, platform::collections::HashMap, prelude::*};

use super::{EffectNodeExecuteState, EffectNodeId};

/// 节点基础设施插件：注册 [`TypedComponentIds`] 资源与节点类型反射。
#[derive(Debug, Default)]
pub struct EffectNodePlugin;

impl Plugin for EffectNodePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TypedComponentIds>()
            .register_type::<EffectNodeExecuteState>()
            .register_type::<EffectNodeId>();
    }
}

/// 主要用于 trigger event 的 component filter：类型 → 组件 ID 映射。
#[derive(Resource, Debug, Default)]
pub struct TypedComponentIds {
    map: HashMap<TypeId, ComponentId>,
}

impl TypedComponentIds {
    /// 注册类型 `T` 的组件 ID。
    pub fn insert<T: Component>(&mut self, component_id: ComponentId) {
        self.map.insert(TypeId::of::<T>(), component_id);
    }

    /// 查询类型 `T` 的组件 ID。
    pub fn get<T: Component>(&self) -> Option<ComponentId> {
        self.map.get(&TypeId::of::<T>()).copied()
    }
}
