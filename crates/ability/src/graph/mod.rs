use bevy::{app::App, prelude::*, reflect::Reflect};
use context::{EffectGraphContext, GraphRef, InstantEffectNodeMap};
use event::{
    trigger_clone_effect_graph_end, trigger_clone_effect_graph_start, trigger_effect_graph_add,
    trigger_effect_graph_exec, trigger_effect_graph_tickable, trigger_effect_graph_to_remove,
    CloneEffectGraphEndEvent, CloneEffectGraphStartEvent, EffectGraphAddEvent,
    EffectGraphExecEvent, EffectGraphRemoveEvent, EffectGraphTickableEvent,
};
use executor::EffectGraphExecutorPlugin;
use graph_map::{EffectGraphBuilderMap, EffectGraphMap};
use state::{update_to_despawn_effect_graph, EffectGraphState, EffectGraphTickState};

use self::state::reset_effect_graph_state;

pub mod blackboard;
pub mod builder;
pub mod bundle;
pub mod context;
pub mod event;
pub mod executor;
pub mod graph_map;
pub mod node;
pub mod pin;
pub mod state;

#[derive(Debug, Default)]
pub struct EffectGraphPlugin;

impl Plugin for EffectGraphPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EffectGraphExecutorPlugin)
            .configure_sets(
                Update,
                (
                    EffectGraphUpdateSystemSet::Execute,
                    EffectGraphUpdateSystemSet::UpdateNode,
                    EffectGraphUpdateSystemSet::UpdateState,
                )
                    .chain(),
            )
            .register_type::<EffectGraphContext>()
            .register_type::<EffectGraphState>()
            .register_type::<EffectGraphTickState>()
            .register_type::<GraphRef>()
            .init_resource::<InstantEffectNodeMap>()
            .init_resource::<EffectGraphMap>()
            .init_resource::<EffectGraphBuilderMap>()
            .add_event::<CloneEffectGraphStartEvent>()
            .add_event::<CloneEffectGraphEndEvent>()
            .add_event::<EffectGraphAddEvent>()
            .add_event::<EffectGraphExecEvent>()
            .add_event::<EffectGraphRemoveEvent>()
            .add_event::<EffectGraphTickableEvent>()
            .add_systems(
                Update,
                reset_effect_graph_state.in_set(EffectGraphUpdateSystemSet::UpdateState),
            )
            .add_systems(Last, update_to_despawn_effect_graph)
            .observe(trigger_clone_effect_graph_start)
            .observe(trigger_clone_effect_graph_end)
            .observe(trigger_effect_graph_add)
            .observe(trigger_effect_graph_exec)
            .observe(trigger_effect_graph_tickable)
            .observe(trigger_effect_graph_to_remove);
    }
}

#[derive(Debug, Default, Component, Reflect, Clone, Copy)]
#[reflect(Component)]
pub struct EffectGraphOwner;

#[derive(SystemSet, Debug, Default, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum EffectGraphUpdateSystemSet {
    #[default]
    Execute,
    UpdateNode,
    UpdateState,
}
