//! Effect 图映射：效果实体 → 图实例引用。

use bevy::{prelude::*, utils::HashMap};

use crate::graph::{
    context::{EffectGraphContext, GraphRef},
    event::{EffectNodeCheckStartEvent, EffectNodeEvent},
};

use super::state::EffectState;

/// 效果实体 → 图实例引用 的资源映射。
#[derive(Debug, Resource, Default, Clone)]
pub struct EffectGraphMap {
    /// 效果实体 → 图引用。
    pub map: HashMap<Entity, GraphRef>,
}

/// 对处于 `CheckCanActive` 的效果向图的入口节点发送开始检查事件。
pub fn effect_graph_check_start(
    graph: Query<&EffectGraphContext>,
    effect_graph_map: Res<EffectGraphMap>,
    query: Query<(&ChildOf, &EffectState)>,
    mut event_writer: EventWriter<EffectNodeCheckStartEvent>,
) {
    for (parent, effect_state) in query.iter() {
        if *effect_state == EffectState::CheckCanActive {
            let graph_context = graph
                .get(
                    effect_graph_map
                        .map
                        .get(&parent.get())
                        .expect("effect graph must exist in map")
                        .get_entity(),
                )
                .expect("effect graph context must exist");

            if let Some(entry_node) = graph_context.entry_node {
                event_writer.send(EffectNodeCheckStartEvent::new(entry_node));
            }
        }
    }
}
