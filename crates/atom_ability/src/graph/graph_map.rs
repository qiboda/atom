//! Effect Graph 图映射：图模板（schema）与图实例的登记与查找。

use bevy::{platform::collections::HashMap, prelude::*, reflect::GetTypeRegistration};

use crate::graph::{builder::EffectGraphBuilder, context::GraphRef};

/// 图类别标识：用于区分不同的技能图。
pub type GraphClass = String;

/// 标记组件：实体是 Effect Graph 模板（schema）。
#[derive(Debug, Component, Default)]
pub struct EffectGraphSchema;

/// 标记组件：实体是 Effect Graph 实例。
#[derive(Debug, Component, Default)]
pub struct EffectGraphInstance;

/// Effect Graph 的资产层：存储技能 Effect Graph 模板，供按类别克隆使用。
#[derive(Debug, Resource, Default)]
pub struct EffectGraphMap {
    /// 图类别 → 图模板引用。
    pub map: HashMap<GraphClass, GraphRef>,
}

impl EffectGraphMap {
    /// 创建空映射。
    pub fn new() -> Self {
        Self {
            map: HashMap::default(),
        }
    }

    /// 登记图模板。
    pub fn insert_graph(&mut self, graph_class: GraphClass, graph: GraphRef) {
        self.map.insert(graph_class, graph);
    }

    /// 按类别查找图模板。
    pub fn get_graph(&self, graph_class: GraphClass) -> Option<GraphRef> {
        self.map.get(&graph_class).copied()
    }
}

/// 图构建器注册表：图类别 → 构建器。
#[derive(Debug, Resource, Default)]
pub struct EffectGraphBuilderMap {
    /// 图类别 → 图构建器。
    pub map: HashMap<GraphClass, Box<dyn EffectGraphBuilder>>,
}

impl EffectGraphBuilderMap {
    /// 按类别查找图构建器。
    pub fn get_effect_graph_builder(&self, name: &str) -> Option<&dyn EffectGraphBuilder> {
        self.map.get(name).map(|x| x.as_ref())
    }
}

/// [`App`] 扩展：支持外部系统注册自定义 [`EffectGraphBuilder`]。
pub trait EffectGraphBuilderMapExt {
    /// 注册类型为 `T` 的图构建器（同时注册其反射类型）。
    fn register_effect_graph_builder<
        T: EffectGraphBuilder + GetTypeRegistration + Default + 'static,
    >(
        &mut self,
    ) -> &mut Self;
}

// support 外部系统添加自定义的 EffectGraphBuilder
impl EffectGraphBuilderMapExt for App {
    fn register_effect_graph_builder<
        T: EffectGraphBuilder + GetTypeRegistration + Default + 'static,
    >(
        &mut self,
    ) -> &mut Self {
        self.register_type::<T>();

        let graph_builder = Box::<T>::default();
        if let Err(e) = self
            .world_mut()
            .get_resource_mut::<EffectGraphBuilderMap>()
            .expect("EffectGraphBuilderMap must insert before insert_effect_graph_builder!")
            .map
            .try_insert(
                graph_builder.get_effect_graph_name().to_string(),
                graph_builder,
            )
        {
            error!("insert_effect_graph_builder error: {}", e)
        }

        self
    }
}
