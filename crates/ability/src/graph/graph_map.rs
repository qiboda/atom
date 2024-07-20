use bevy::{prelude::*, reflect::GetTypeRegistration, utils::HashMap};

use crate::graph::{builder::EffectGraphBuilder, context::GraphRef};

pub type GraphClass = String;

#[derive(Debug, Component, Default)]
pub struct EffectGraphSchema;

#[derive(Debug, Component, Default)]
pub struct EffectGraphInstance;

/// Effect Graph的资产层
/// 存储技能Effect Graph，作为模板使用。
#[derive(Debug, Resource, Default)]
pub struct EffectGraphMap {
    pub map: HashMap<GraphClass, GraphRef>,
}

impl EffectGraphMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::default(),
        }
    }

    pub fn insert_graph(&mut self, graph_class: GraphClass, graph: GraphRef) {
        self.map.insert(graph_class, graph);
    }

    pub fn get_graph(&self, graph_class: GraphClass) -> Option<GraphRef> {
        self.map.get(&graph_class).copied()
    }
}

#[derive(Debug, Resource, Default)]
pub struct EffectGraphBuilderMap {
    pub map: HashMap<GraphClass, Box<dyn EffectGraphBuilder>>,
}

impl EffectGraphBuilderMap {
    pub fn get_effect_graph_builder(&self, name: &str) -> Option<&dyn EffectGraphBuilder> {
        self.map.get(name).map(|x| x.as_ref())
    }
}

pub trait EffectGraphBuilderMapExt {
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
