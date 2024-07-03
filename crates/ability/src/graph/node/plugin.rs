use std::any::TypeId;

use bevy::{app::Plugin, ecs::component::ComponentId, prelude::*, utils::HashMap};

use super::{EffectNodeExecuteState, EffectNodeId};

#[derive(Debug, Default)]
pub struct EffectNodePlugin;

impl Plugin for EffectNodePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TypedComponentIds>()
            .register_type::<EffectNodeExecuteState>()
            .register_type::<EffectNodeId>();
    }
}

/// 主要用于trigger event的component filter。
#[derive(Resource, Debug, Default)]
pub struct TypedComponentIds {
    map: HashMap<TypeId, ComponentId>,
}

impl TypedComponentIds {
    pub fn insert<T: Component>(&mut self, component_id: ComponentId) {
        self.map.insert(TypeId::of::<T>(), component_id);
    }

    pub fn get<T: Component>(&self) -> Option<ComponentId> {
        self.map.get(&TypeId::of::<T>()).copied()
    }
}
