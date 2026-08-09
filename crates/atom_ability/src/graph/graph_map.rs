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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::context::InstantEffectNodeMap;

    #[test]
    fn new_graph_map_is_empty() {
        let map = EffectGraphMap::new();
        assert!(map.map.is_empty());
    }

    #[test]
    fn insert_graph_then_get_roundtrip() {
        let mut map = EffectGraphMap::new();
        let graph = GraphRef::new(Entity::from_bits(42));

        assert_eq!(map.get_graph("fireball".to_string()), None);
        map.insert_graph("fireball".to_string(), graph);

        assert_eq!(map.get_graph("fireball".to_string()), Some(graph));
    }

    #[test]
    fn insert_graph_overwrites_previous() {
        let mut map = EffectGraphMap::new();
        map.insert_graph("fireball".to_string(), GraphRef::new(Entity::from_bits(1)));

        let replacement = GraphRef::new(Entity::from_bits(2));
        map.insert_graph("fireball".to_string(), replacement);

        assert_eq!(map.get_graph("fireball".to_string()), Some(replacement));
    }

    #[test]
    fn get_graph_missing_class_returns_none() {
        let map = EffectGraphMap::new();
        assert_eq!(map.get_graph("nonexistent".to_string()), None);
    }

    #[derive(Debug, Default)]
    struct TestGraphBuilder;

    impl EffectGraphBuilder for TestGraphBuilder {
        fn get_effect_graph_name(&self) -> &'static str {
            "test_graph"
        }

        fn build(
            &self,
            _commands: &mut Commands,
            _instant_map: &mut ResMut<InstantEffectNodeMap>,
        ) -> Entity {
            Entity::PLACEHOLDER
        }
    }

    #[test]
    fn builder_map_get_returns_registered_builder() {
        let mut map = EffectGraphBuilderMap::default();
        map.map.insert(
            "test_graph".to_string(),
            Box::new(TestGraphBuilder) as Box<dyn EffectGraphBuilder>,
        );

        let builder = map
            .get_effect_graph_builder("test_graph")
            .expect("已注册的构建器应可查到");
        assert_eq!(builder.get_effect_graph_name(), "test_graph");
    }

    #[test]
    fn builder_map_get_missing_name_returns_none() {
        let map = EffectGraphBuilderMap::default();
        assert!(map.get_effect_graph_builder("missing").is_none());
    }
}
