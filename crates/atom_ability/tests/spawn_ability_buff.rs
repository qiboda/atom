//! RED 回归测试：Bundle → BSN 迁移（issue #7）后模板函数的场景产物断言。
//!
//! 对应计划 `.omo/plans/bsn-migration.md` §4.2（ability 侧）/§4.3（buff 侧）与验收 §5。
//! 迁移前 `AbilityBundle::new`/`BuffBundle::new` 构造出的实体组件集合必须与
//! `spawn_ability`/`spawn_buff` 场景产物**完全一致**——本文件把这些行为固化为回归测试。
//!
//! 目标 API（迁移后应存在，当前不存在 → 编译失败 = RED）：
//! - `atom_ability::ability::bundle::spawn_ability(config, &registry) -> impl Scene`
//! - `atom_ability::ability::bundle::spawn_ability_owner::<T>() -> impl Scene`
//! - `atom_ability::buff::bundle::spawn_buff(config, &registry) -> impl Scene`

use atom_ability::{
    ability::bundle::AbilityConfigData,
    buff::bundle::BuffConfigData,
    config::{AbilityConfig, AbilityType, BuffConfig, RevertableLayerTag},
};
use atom_ability::{
    ability::{
        bundle::{spawn_ability, spawn_ability_owner},
        comp::{Ability, AbilityExecuteState, AbilityTickState},
        layertag::tag::{
            AbilityAbortDisableLayerTagContainer, AbilityAbortRequiredLayerTagContainer,
            AbilityAddedLayerTagContainer, AbilityLayerTagContainerRevert,
            AbilityRemovedLayerTagContainer, AbilityStartDisableLayerTagContainer,
            AbilityStartRequiredLayerTagContainer,
        },
    },
    attribute::{Attribute, attribute_set::AttributeSet, implement::attr_base::ValueAttribute},
    buff::{
        bundle::spawn_buff,
        layer::BuffLayer,
        layertag::tag::{
            BuffAbortDisableLayerTagContainer, BuffAbortRequiredLayerTagContainer,
            BuffAddedLayerTagContainer, BuffLayerTagContainerRevert, BuffRemovedLayerTagContainer,
            BuffStartDisableLayerTagContainer, BuffStartRequiredLayerTagContainer,
        },
        state::{Buff, BuffExecuteState, BuffTickState},
        timer::BuffTime,
    },
    graph::EffectGraphOwner,
    stateset::{StateLayerTagContainer, StateLayerTagRegistry},
};
use atom_layertag::container_op::LayerTagContainer;
use bevy::{MinimalPlugins, asset::AssetPlugin, prelude::*, scene::ScenePlugin};

/// 构造带混合（已注册 + 未注册）标签的测试技能数据行。
fn make_ability_config() -> AbilityConfig {
    AbilityConfig {
        id: 1,
        name: "测试技能".to_string(),
        desc: String::new(),
        graph_class: "test_graph".to_string(),
        activation_type: AbilityType::Active,
        cd: 5.0,
        start_required_layertags: vec![
            "start_required_a".to_string(),
            // 故意不注册：迁移后必须跳过（warn）而非 panic
            "start_required_missing".to_string(),
        ],
        start_disabled_layertags: vec!["start_disabled_a".to_string()],
        start_added_layertags: vec![RevertableLayerTag {
            raw_layertag: "start_added_a".to_string(),
            revertable: true,
        }],
        start_removed_layertags: vec![RevertableLayerTag {
            raw_layertag: "start_removed_a".to_string(),
            revertable: false,
        }],
        abort_required_layertags: vec!["abort_required_a".to_string()],
        abort_disabled_layertags: vec!["abort_disabled_a".to_string()],
    }
}

/// 构造 buff 测试数据行（interval > 0，应有 looper 计时器）。
fn make_buff_config() -> BuffConfig {
    BuffConfig {
        id: 2,
        name: "测试buff".to_string(),
        desc: String::new(),
        graph_class: "test_graph".to_string(),
        max_layer: 3,
        duration: 10.0,
        interval: 1.5,
        start_required_layertags: vec!["buff_start_required_a".to_string()],
        start_disabled_layertags: vec!["buff_start_disabled_a".to_string()],
        start_added_layertags: vec![RevertableLayerTag {
            raw_layertag: "buff_start_added_a".to_string(),
            revertable: true,
        }],
        start_removed_layertags: vec![RevertableLayerTag {
            raw_layertag: "buff_start_removed_a".to_string(),
            revertable: false,
        }],
        abort_required_layertags: vec!["buff_abort_required_a".to_string()],
        abort_disabled_layertags: vec!["buff_abort_disabled_a".to_string()],
    }
}

/// 构造 interval = 0 的 buff 数据行（迁移后不应有 looper 计时器）。
fn make_buff_config_no_interval() -> BuffConfig {
    BuffConfig {
        id: 3,
        name: "无周期buff".to_string(),
        desc: String::new(),
        graph_class: "test_graph".to_string(),
        max_layer: 1,
        duration: 3.0,
        interval: 0.0,
        start_required_layertags: vec![],
        start_disabled_layertags: vec![],
        start_added_layertags: vec![],
        start_removed_layertags: vec![],
        abort_required_layertags: vec![],
        abort_disabled_layertags: vec![],
    }
}

/// 构造带全部未注册标签的 buff 数据行（验证跳过而非 panic）。
fn make_buff_config_all_unregistered() -> BuffConfig {
    BuffConfig {
        id: 4,
        name: "全未注册buff".to_string(),
        desc: String::new(),
        graph_class: "test_graph".to_string(),
        max_layer: 1,
        duration: 1.0,
        interval: 0.0,
        start_required_layertags: vec!["ghost_tag".to_string()],
        start_disabled_layertags: vec!["ghost_tag".to_string()],
        start_added_layertags: vec![],
        start_removed_layertags: vec![],
        abort_required_layertags: vec!["ghost_tag".to_string()],
        abort_disabled_layertags: vec!["ghost_tag".to_string()],
    }
}

/// 注册测试所需的全部 layertag 原始标签。
fn register_test_layertags(app: &mut App) {
    let mut registry = app.world_mut().resource_mut::<StateLayerTagRegistry>();
    for tag in [
        "start_required_a",
        "start_disabled_a",
        "start_added_a",
        "start_removed_a",
        "abort_required_a",
        "abort_disabled_a",
        "buff_start_required_a",
        "buff_start_disabled_a",
        "buff_start_added_a",
        "buff_start_removed_a",
        "buff_abort_required_a",
        "buff_abort_disabled_a",
    ] {
        registry.0.register_raw(tag);
    }
}

fn scene_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), ScenePlugin))
        .insert_resource(StateLayerTagRegistry::default());
    app
}

#[test]
fn spawn_ability_produces_full_ability_entity() {
    let mut app = scene_app();
    register_test_layertags(&mut app);
    let config = make_ability_config();
    app.add_systems(
        Update,
        move |mut commands: Commands, registry: Res<StateLayerTagRegistry>| {
            let scene = spawn_ability(&config, &registry);
            commands.spawn_scene(scene);
        },
    );
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(
        &Ability,
        &AbilityExecuteState,
        &AbilityTickState,
        &AbilityConfigData,
        &EffectGraphOwner,
        &AbilityStartRequiredLayerTagContainer,
        &AbilityStartDisableLayerTagContainer,
        &AbilityAddedLayerTagContainer,
        &AbilityRemovedLayerTagContainer,
        &AbilityAbortRequiredLayerTagContainer,
        &AbilityAbortDisableLayerTagContainer,
    )>();
    let item = query
        .iter(world)
        .next()
        .expect("spawn_ability 场景应产生一个技能实体");

    let (
        _ability,
        execute_state,
        tick_state,
        ability_row,
        _graph_owner,
        start_required,
        start_disabled,
        added,
        removed,
        abort_required,
        abort_disabled,
    ) = item;

    // 构造逻辑与迁移前 AbilityBundle::new 一致：默认状态。
    assert_eq!(*execute_state, AbilityExecuteState::Inactive);
    assert_eq!(*tick_state, AbilityTickState::Ticked);
    assert_eq!(
        ability_row.graph_class, "test_graph",
        "配置数据组件的图类别必须与输入一致"
    );

    // 6 个 layertag 容器：已注册标签解析成功、未注册标签被跳过（不 panic）。
    assert!(
        start_required
            .0
            .iter_layertag()
            .any(|t| t.raw_layertag() == "start_required_a")
    );
    assert!(
        !start_required
            .0
            .iter_layertag()
            .any(|t| t.raw_layertag() == "start_required_missing"),
        "未注册 layertag 应被跳过而非 panic"
    );
    assert!(
        start_disabled
            .0
            .iter_layertag()
            .any(|t| t.raw_layertag() == "start_disabled_a")
    );
    assert!(
        added
            .layer_tag_container
            .iter_layertag()
            .any(|t| t.raw_layertag() == "start_added_a")
    );
    assert_eq!(added.revert, AbilityLayerTagContainerRevert::Yes);
    assert!(
        removed
            .layer_tag_container
            .iter_layertag()
            .any(|t| t.raw_layertag() == "start_removed_a")
    );
    assert_eq!(removed.revert, AbilityLayerTagContainerRevert::No);
    assert!(
        abort_required
            .0
            .iter_layertag()
            .any(|t| t.raw_layertag() == "abort_required_a")
    );
    assert!(
        abort_disabled
            .0
            .iter_layertag()
            .any(|t| t.raw_layertag() == "abort_disabled_a")
    );
}

#[test]
fn spawn_ability_skips_all_unregistered_layertags_without_panicking() {
    let mut app = scene_app();
    // 不注册任何 layertag：全部原始标签缺失。
    let config = AbilityConfig {
        id: 1,
        name: "全未注册技能".to_string(),
        desc: String::new(),
        graph_class: "test_graph".to_string(),
        activation_type: AbilityType::Active,
        cd: 5.0,
        start_required_layertags: vec!["ghost_a".to_string()],
        start_disabled_layertags: vec!["ghost_a".to_string()],
        start_added_layertags: vec![],
        start_removed_layertags: vec![],
        abort_required_layertags: vec!["ghost_a".to_string()],
        abort_disabled_layertags: vec!["ghost_a".to_string()],
    };

    app.add_systems(
        Update,
        move |mut commands: Commands, registry: Res<StateLayerTagRegistry>| {
            let scene = spawn_ability(&config, &registry);
            commands.spawn_scene(scene);
        },
    );
    // 未注册标签必须跳过（warn）而非 panic。
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(
        &AbilityStartRequiredLayerTagContainer,
        &AbilityAbortRequiredLayerTagContainer,
    )>();
    let (start_required, abort_required) = query
        .iter(world)
        .next()
        .expect("spawn_ability 场景应产生一个技能实体");
    assert!(
        start_required.0.iter_layertag().next().is_none(),
        "未注册标签不应进入容器"
    );
    assert!(abort_required.0.iter_layertag().next().is_none());
}

/// 测试用属性集（满足 `T: AttributeSet + Component + Default + Clone` 约束——BSN
/// 场景模板通过 `FromTemplate`（`Clone + Default + Unpin`）注入泛型组件）。
#[derive(Component, Debug, Default, Clone)]
struct TestAttributeSet {
    hp: Box<ValueAttribute>,
}

/// 测试用属性集枚举成员。
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)] // 测试仅验证枚举类型存在，不构造具体成员。
enum TestAttributeSetType {
    /// 生命值。
    Hp,
}

impl AttributeSet for TestAttributeSet {
    type AttributeSetEnum = TestAttributeSetType;

    fn get_attr_final_value(&self, _attribute_set_enum: Self::AttributeSetEnum) -> Option<f32> {
        None
    }

    fn get_attr(&self, _attribute_set_enum: Self::AttributeSetEnum) -> &dyn Attribute {
        self.hp.as_ref()
    }

    fn get_attr_mut(&mut self, _attribute_set_enum: Self::AttributeSetEnum) -> &mut dyn Attribute {
        self.hp.as_mut()
    }
}

#[test]
fn spawn_ability_owner_produces_attribute_set_and_state_container() {
    let mut app = scene_app();
    app.add_systems(Update, |mut commands: Commands| {
        let scene = spawn_ability_owner::<TestAttributeSet>();
        commands.spawn_scene(scene);
    });
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(&TestAttributeSet, &StateLayerTagContainer)>();
    let (_attribute_set, _state_container) = query
        .iter(world)
        .next()
        .expect("spawn_ability_owner 场景应产生一个持有者实体");
}

#[test]
fn spawn_buff_produces_full_buff_entity() {
    let mut app = scene_app();
    register_test_layertags(&mut app);
    let config = make_buff_config();
    app.add_systems(
        Update,
        move |mut commands: Commands, registry: Res<StateLayerTagRegistry>| {
            let scene = spawn_buff(&config, &registry);
            commands.spawn_scene(scene);
        },
    );
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(
        &Buff,
        &EffectGraphOwner,
        &BuffExecuteState,
        &BuffTickState,
        &BuffTime,
        &BuffLayer,
        &BuffConfigData,
        &BuffStartRequiredLayerTagContainer,
        &BuffStartDisableLayerTagContainer,
        &BuffAddedLayerTagContainer,
        &BuffRemovedLayerTagContainer,
        &BuffAbortRequiredLayerTagContainer,
        &BuffAbortDisableLayerTagContainer,
    )>();
    let item = query
        .iter(world)
        .next()
        .expect("spawn_buff 场景应产生一个 buff 实体");

    let (
        _buff,
        _graph_owner,
        execute_state,
        tick_state,
        buff_time,
        buff_layer,
        buff_row,
        start_required,
        start_disabled,
        added,
        removed,
        abort_required,
        abort_disabled,
    ) = item;

    // 构造逻辑与迁移前 BuffBundle::new 一致。
    assert_eq!(*execute_state, BuffExecuteState::Inactive);
    assert_eq!(*tick_state, BuffTickState::Ticked);
    assert_eq!(
        buff_row.graph_class, "test_graph",
        "配置数据组件的图类别必须与输入一致"
    );

    // BuffTime：duration 与 interval 来自 row.data()。
    assert_eq!(
        buff_time.once_timer.duration().as_secs_f32(),
        10.0,
        "once_timer 时长必须等于数据表 duration"
    );
    let looper = buff_time
        .looper_timer
        .as_ref()
        .expect("interval > 0 时必须创建 looper 计时器");
    assert_eq!(
        looper.duration().as_secs_f32(),
        1.5,
        "looper 周期必须等于数据表 interval"
    );

    // BuffLayer：max_layer 来自 row.data()，初始 1 层。
    assert_eq!(
        buff_layer.get_field::<i32>("max_layer"),
        Some(&3),
        "max_layer 必须来自数据表行"
    );
    assert_eq!(buff_layer.get_field::<i32>("layer"), Some(&1));

    // 6 个 buff layertag 容器。
    assert!(
        start_required
            .0
            .iter_layertag()
            .any(|t| t.raw_layertag() == "buff_start_required_a")
    );
    assert!(
        start_disabled
            .0
            .iter_layertag()
            .any(|t| t.raw_layertag() == "buff_start_disabled_a")
    );
    assert!(
        added
            .layer_tag_container
            .iter_layertag()
            .any(|t| t.raw_layertag() == "buff_start_added_a")
    );
    assert_eq!(added.revert, BuffLayerTagContainerRevert::Yes);
    assert!(
        removed
            .layer_tag_container
            .iter_layertag()
            .any(|t| t.raw_layertag() == "buff_start_removed_a")
    );
    assert_eq!(removed.revert, BuffLayerTagContainerRevert::No);
    assert!(
        abort_required
            .0
            .iter_layertag()
            .any(|t| t.raw_layertag() == "buff_abort_required_a")
    );
    assert!(
        abort_disabled
            .0
            .iter_layertag()
            .any(|t| t.raw_layertag() == "buff_abort_disabled_a")
    );
}

#[test]
fn spawn_buff_with_zero_interval_has_no_looper() {
    let mut app = scene_app();
    let config = make_buff_config_no_interval();
    app.add_systems(
        Update,
        move |mut commands: Commands, registry: Res<StateLayerTagRegistry>| {
            let scene = spawn_buff(&config, &registry);
            commands.spawn_scene(scene);
        },
    );
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(&BuffTime, &BuffConfigData)>();
    let (buff_time, buff_row) = query
        .iter(world)
        .next()
        .expect("spawn_buff 场景应产生一个 buff 实体");
    assert_eq!(buff_row.graph_class, "test_graph");
    assert!(
        buff_time.looper_timer.is_none(),
        "interval <= 0 时不应创建 looper 计时器"
    );
    assert_eq!(buff_time.once_timer.duration().as_secs_f32(), 3.0);
}

#[test]
fn spawn_buff_skips_all_unregistered_layertags_without_panicking() {
    let mut app = scene_app();
    // 不注册任何 layertag。
    let config = make_buff_config_all_unregistered();
    app.add_systems(
        Update,
        move |mut commands: Commands, registry: Res<StateLayerTagRegistry>| {
            let scene = spawn_buff(&config, &registry);
            commands.spawn_scene(scene);
        },
    );
    // 未注册标签必须跳过（warn）而非 panic。
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(
        &BuffStartRequiredLayerTagContainer,
        &BuffAbortRequiredLayerTagContainer,
    )>();
    let (start_required, abort_required) = query
        .iter(world)
        .next()
        .expect("spawn_buff 场景应产生一个 buff 实体");
    assert!(start_required.0.iter_layertag().next().is_none());
    assert!(abort_required.0.iter_layertag().next().is_none());
}
