//! RED 集成测试（核心交付）：grant_effect 节点重建后的反射调度全链路。
//!
//! 对应计划 `.omo/plans/bsn-migration.md` §4.4（Q2 修订：保留 `EffectBundleTrait`
//! 反射调度 + 新架构重建 grant_effect）与 §6 测试策略。
//!
//! 链路：`EffectValue::BoxReflect(bundle)` → 输入 pin → `EffectNodeExecEvent` →
//! observer 经 `AppTypeRegistry` 取 `ReflectEffectBundleTrait` → `spawn_scene` → 新实体。
//!
//! 目标 API（迁移后应存在，当前不存在 → 编译失败 = RED）：
//! - `atom_ability::bundle::{EffectBundleTrait, ReflectEffectBundleTrait}`（Q4）
//! - `atom_ability::graph::blackboard::EffectValue::BoxReflect`
//! - `atom_ability::graph::node::implement::grant_effect::{EffectNodeGrantEffect, EffectNodeGrantEffectPlugin}`
//!
//! 注：module 路径按计划 §4.4 首选方案 `node/implement/grant_effect.rs`（与 log/timer/seq 同层）。

use std::any::TypeId;

use atom_ability::{
    bundle::{EffectBundleTrait, ReflectEffectBundleTrait},
    graph::{
        EffectGraphPlugin,
        blackboard::EffectValue,
        context::EffectGraphContext,
        event::EffectNodeExecEvent,
        executor::EffectGraphExecutor,
        node::{
            EffectNodeExecuteState, EffectNodeId,
            implement::grant_effect::{EffectNodeGrantEffect, EffectNodeGrantEffectPlugin},
            pin::EffectNodeSlot,
        },
        pin::{EffectNodeExecPin, EffectNodeSlotPin, EffectNodeSlotValue},
    },
};
use bevy::{
    MinimalPlugins, asset::AssetPlugin, ecs::reflect::AppTypeRegistry, prelude::*,
    scene::ScenePlugin,
};

/// 测试效果组件：由 `TestEffectBundle::build_scene` 的场景产生。
///
/// `Default + Clone` 触发 blanket `FromTemplate`，可直接在 `bsn!` 中以字段形式构造。
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
struct TestEffectComponent {
    damage: f32,
}

/// 测试用具体效果 bundle：实现 `EffectBundleTrait` + 反射注册（Q2/Q4 核心）。
#[derive(Debug, Clone, Reflect)]
#[reflect(EffectBundleTrait)]
struct TestEffectBundle {
    damage: f32,
}

impl EffectBundleTrait for TestEffectBundle {
    fn build_scene(&self) -> Box<dyn Scene> {
        let damage = self.damage;
        Box::new(bsn! { TestEffectComponent { damage: {damage} } })
    }
}

#[test]
fn grant_effect_reflect_dispatch_spawns_effect_scene() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        ScenePlugin::default(),
        // 初始化 InstantEffectNodeMap 等图资源（observer 的 system param 需要）。
        EffectGraphPlugin,
        EffectNodeGrantEffectPlugin,
    ))
    // 注册具体 bundle 类型：同时注册 Reflect + ReflectEffectBundleTrait 类型数据。
    .register_type::<TestEffectBundle>();

    // 盲区探测：反射调度依赖的类型数据必须真实存在（register_type 生效）。
    {
        let registry = app.world().resource::<AppTypeRegistry>();
        let read = registry.read();
        assert!(
            read.get_type_data::<ReflectEffectBundleTrait>(TypeId::of::<TestEffectBundle>())
                .is_some(),
            "register_type 后必须能从 AppTypeRegistry 取到 ReflectEffectBundleTrait"
        );
    }

    // 搭建图：根实体（context + executor）与子节点（grant_effect + 执行状态）。
    let graph_entity = app
        .world_mut()
        .spawn((EffectGraphContext::new(), EffectGraphExecutor::default()))
        .id();
    let node_entity = app
        .world_mut()
        .spawn((
            EffectNodeGrantEffect::default(),
            EffectNodeExecuteState::default(),
        ))
        .set_parent_in_place(graph_entity)
        .id();

    // 输入 pin 写入 BoxReflect 值：start => (effect_bundle: Box<dyn EffectBundleTrait>)。
    {
        let world = app.world_mut();
        let mut entity = world.entity_mut(graph_entity);
        let mut context = entity
            .get_mut::<EffectGraphContext>()
            .expect("图根实体必须有 EffectGraphContext");
        context.insert_input_value(
            EffectNodeSlotPin {
                node_id: EffectNodeId::Entity(node_entity),
                slot: EffectNodeSlot::new::<Box<dyn EffectBundleTrait>>(
                    EffectNodeGrantEffect::INPUT_SLOT_EFFECT_BUNDLE,
                ),
            },
            EffectValue::BoxReflect(Box::new(TestEffectBundle { damage: 5.0 })).into(),
        );
    }

    // 触发节点执行（仿 timer 的触发方式）。
    app.add_systems(Update, move |mut commands: Commands| {
        commands.trigger(EffectNodeExecEvent {
            input_exec_pin: EffectNodeExecPin {
                node_id: EffectNodeId::Entity(node_entity),
                exec: EffectNodeGrantEffect::INPUT_EXEC_START.into(),
            },
        });
    });
    app.update();

    // 断言 1：node.effects 非空（observer 记录了生成的实体）。
    let effect_entity = {
        let world = app.world();
        let node = world
            .entity(node_entity)
            .get::<EffectNodeGrantEffect>()
            .expect("节点实体必须有 EffectNodeGrantEffect 组件");
        assert!(
            !node.effects.is_empty(),
            "grant_effect 执行后 effects 必须非空"
        );
        node.effects[0]
    };

    // 断言 2：新实体具有 TestEffectComponent，且值正确（反射调度 → spawn_scene 全链路）。
    {
        let world = app.world();
        let component = world
            .entity(effect_entity)
            .get::<TestEffectComponent>()
            .expect("grant_effect 生成的实体必须带有 TestEffectComponent");
        assert_eq!(component.damage, 5.0, "组件值必须来自 bundle 数据");
    }

    // 断言 3：输出 pin start_effect_entity 写入了生成的实体。
    {
        let world = app.world();
        let context = world
            .entity(graph_entity)
            .get::<EffectGraphContext>()
            .expect("图根实体必须有 EffectGraphContext");
        let output = context.get_output_value(&EffectNodeSlotPin {
            node_id: EffectNodeId::Entity(node_entity),
            slot: EffectNodeSlot::new::<Entity>(
                EffectNodeGrantEffect::OUTPUT_SLOT_START_EFFECT_ENTITY,
            ),
        });
        match output {
            Some(EffectNodeSlotValue::Value(EffectValue::Entity(entity))) => {
                assert_eq!(*entity, effect_entity, "输出 pin 必须指向生成的实体");
            }
            _ => panic!("输出 pin start_effect_entity 必须写入 EffectValue::Entity"),
        }
    }
}
