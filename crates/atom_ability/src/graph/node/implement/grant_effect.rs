//! 效果授予节点：从输入 pin 的反射 bundle 生成效果实体。

use std::ops::{Deref, Not};

use bevy::{ecs::reflect::AppTypeRegistry, prelude::*};

use crate::{
    bundle::{EffectBundleTrait, ReflectEffectBundleTrait},
    graph::{
        blackboard::EffectValue,
        context::{EffectGraphContext, InstantEffectNodeMap},
        event::EffectNodeExecEvent,
        executor::EffectGraphExecutor,
        node::{EffectNode, EffectNodeExecuteState, EffectNodeId, pin::EffectNodePinGroup},
        pin::{EffectNodeExecPin, EffectNodeSlotPin, EffectNodeSlotValue},
    },
    impl_effect_node_pin_group,
};

/// 效果授予节点插件：注册类型反射 + 挂载执行 observer。
#[derive(Debug, Default)]
pub struct EffectNodeGrantEffectPlugin;

impl Plugin for EffectNodeGrantEffectPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EffectNodeGrantEffect>()
            .add_observer(trigger_effect_node_grant_effect);
    }
}

/// 效果授予节点：记录已生成的效果实体。
#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct EffectNodeGrantEffect {
    /// 已生成的效果实体列表。
    pub effects: Vec<Entity>,
}

impl_effect_node_pin_group!(EffectNodeGrantEffect,
    input => (
        start => (effect_bundle: Box<dyn EffectBundleTrait>)
    )
    output => (
        start => (start_effect_entity: Entity),
        finish => (end_effect_entity: Entity)
    )
);

impl EffectNode for EffectNodeGrantEffect {}

fn trigger_effect_node_grant_effect(
    trigger: On<EffectNodeExecEvent>,
    mut commands: Commands,
    mut query: Query<(
        &mut EffectNodeGrantEffect,
        &mut EffectNodeExecuteState,
        &ChildOf,
    )>,
    mut graph_query: Query<(&mut EffectGraphContext, &mut EffectGraphExecutor)>,
    instant_nodes: Res<InstantEffectNodeMap>,
    type_registry: Res<AppTypeRegistry>,
) {
    let pin = trigger.event().input_exec_pin;
    let EffectNodeId::Entity(entity) = pin.node_id else {
        return;
    };

    if let Ok((mut node, mut state, parent)) = query.get_mut(entity) {
        info!("trigger_node_event: grant_effect {:?}", pin);

        if let Ok((mut context, mut executor)) = graph_query.get_mut(parent.parent())
            && pin.exec.name == EffectNodeGrantEffect::INPUT_EXEC_START
        {
            // 1. 读取输入 pin 的 BoxReflect 值
            let Some(EffectNodeSlotValue::Value(EffectValue::BoxReflect(v))) = context
                .get_input_value(&EffectNodeSlotPin {
                    node_id: EffectNodeId::Entity(entity),
                    slot: *node
                        .get_input_slot_pin_by_name(EffectNodeGrantEffect::INPUT_SLOT_EFFECT_BUNDLE)
                        .expect("effect_bundle 输入槽必须存在"),
                })
            else {
                return;
            };

            // 2. 反射调度：经 AppTypeRegistry 取 ReflectEffectBundleTrait，downcast 到 &dyn EffectBundleTrait
            let read_guard = type_registry.read();
            let reflect_a_trait = read_guard
                .get_type_data::<ReflectEffectBundleTrait>(v.type_id())
                .expect("ReflectEffectBundleTrait must be registered for the value type");
            let effect_bundle: &dyn EffectBundleTrait = reflect_a_trait
                .get(v.deref())
                .expect("value must be an EffectBundle");

            // 3. spawn_scene（Q4 语义）生成效果实体
            let effect_entity = effect_bundle.spawn_scene(&mut commands).id();
            node.effects.push(effect_entity);

            // 4. 写入输出 pin start_effect_entity 并推进 start 执行链
            context.insert_output_value(
                EffectNodeSlotPin {
                    node_id: EffectNodeId::Entity(entity),
                    slot: *node
                        .get_output_slot_pin_by_name(
                            EffectNodeGrantEffect::OUTPUT_SLOT_START_EFFECT_ENTITY,
                        )
                        .expect("start_effect_entity 输出槽必须存在"),
                },
                EffectValue::Entity(effect_entity).into(),
            );

            if node.effects.is_empty().not() {
                *state = EffectNodeExecuteState::Active;
            }

            executor.start_push_output_pin(
                EffectNodeExecPin {
                    node_id: entity.into(),
                    exec: EffectNodeGrantEffect::OUTPUT_EXEC_START.into(),
                },
                &context,
                &instant_nodes,
            );
        }
    }
}
