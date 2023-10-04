use bevy::{prelude::*, utils::HashMap};

use crate::graph::{
    context::{EffectGraphContext, GraphRef},
    event::{EffectNodeCheckStartEvent, EffectNodeEvent},
};

use super::state::EffectState;

#[derive(Debug, Resource, Default, Clone)]
pub struct EffectGraphMap {
    pub map: HashMap<Entity, GraphRef>,
}

pub fn effect_graph_check_start(
    graph: Query<&EffectGraphContext>,
    effect_graph_map: Res<EffectGraphMap>,
    query: Query<(&Parent, &EffectState)>,
    mut event_writer: EventWriter<EffectNodeCheckStartEvent>,
) {
    for (parent, effect_state) in query.iter() {
        if *effect_state == EffectState::CheckCanActive {
            let graph_context = graph
                .get(
                    effect_graph_map
                        .map
                        .get(&parent.get())
                        .unwrap()
                        .get_entity(),
                )
                .unwrap();

            if let Some(entry_node) = graph_context.entry_node {
                event_writer.send(EffectNodeCheckStartEvent::new(entry_node));
            }
        }
    }
}
