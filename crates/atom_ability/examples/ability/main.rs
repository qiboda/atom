mod attribute;
mod base_attack;

use atom_ability::{
    AbilitySubsystemPlugin,
    ability::{
        bundle::{spawn_ability, spawn_ability_owner},
        comp::Ability,
        event::{AbilityRemoveEvent, AbilityStartEvent},
    },
    buff::node::buff_entry::EffectNodeBuffEntryPlugin,
    config::AbilityConfig,
    graph::{
        graph_map::EffectGraphBuilderMapExt,
        node::implement::{
            log::EffectNodeLogPlugin, seq::EffectNodeSeqPlugin, timer::EffectNodeTimerPlugin,
        },
    },
    stateset::StateLayerTagRegistry,
};

use attribute::BaseAttributeSet;
use base_attack::EffectNodeGraphBaseAttack;

use atom_data::{DataRegistry, DataRegistryPlugin, DataTable};
use bevy::{DefaultPlugins, asset::AssetServer, input::ButtonInput, log::info, prelude::*};
use bevy_common_assets::json::JsonAssetPlugin;

#[derive(Component, Reflect)]
struct Player;

fn main() {
    dotenv::dotenv().ok();

    let mut app = App::new();
    app.insert_resource(LoadedTables::default());
    app.add_plugins(DefaultPlugins)
        .add_plugins(JsonAssetPlugin::<DataTable<AbilityConfig>>::new(&["json"]))
        .add_plugins(DataRegistryPlugin)
        .add_plugins(AbilitySubsystemPlugin)
        .add_plugins(EffectNodeTimerPlugin)
        .add_plugins(EffectNodeLogPlugin)
        .add_plugins(EffectNodeSeqPlugin)
        .add_plugins(EffectNodeBuffEntryPlugin);
    DataRegistryPlugin::register_table::<AbilityConfig>(&mut app);
    app.register_effect_graph_builder::<EffectNodeGraphBaseAttack>()
        .add_systems(Startup, load_ability_table)
        .add_systems(Update, create_ability)
        .add_systems(Update, cast_base_skill)
        .add_systems(Update, remove_base_skill)
        .run();
}

/// 从 JSON 数据表加载技能配置（`assets/datatables/AbilityConfig.json`）。
///
/// handle 存入 [`LoadedTables`] 资源保持存活——`Assets::track_assets` 会移除无强引用的
/// 资产，若丢弃 handle，表可能在 `sync_table` 读取前被回收（`#[must_use]` 契约，见
/// `atom_data::DataRegistry::load` 文档）。
fn load_ability_table(
    mut registry: ResMut<DataRegistry>,
    server: Res<AssetServer>,
    mut loaded: ResMut<LoadedTables>,
) {
    loaded
        .tables
        .push(registry.load::<AbilityConfig>(&server, "datatables/AbilityConfig.json"));
}

/// 已加载数据表 handle 集合（保持资产存活，防 track_assets 提前回收）。
#[derive(Resource, Default)]
struct LoadedTables {
    tables: Vec<Handle<DataTable<AbilityConfig>>>,
}

fn create_ability(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    registry: Res<DataRegistry>,
    state_registry: Res<StateLayerTagRegistry>,
    query: Query<(), With<Ability>>,
) {
    if input.just_pressed(KeyCode::KeyC) {
        if query.iter().count() > 0 {
            return;
        }

        let Some(config) = registry.get::<AbilityConfig>(&1) else {
            return;
        };

        let owner = commands
            .spawn_scene(spawn_ability_owner::<BaseAttributeSet>())
            .insert(Player)
            .id();
        commands
            .spawn_scene(spawn_ability(config, &state_registry))
            .set_parent_in_place(owner);

        info!("create_ability");
    }
}

/// only can cast once, because node has not reset state.
fn cast_base_skill(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    ability_query: Query<Entity, With<Ability>>,
) {
    if input.just_pressed(KeyCode::KeyQ) {
        info!("just_pressed: cast_base_skill");
        for entity in ability_query.iter() {
            commands.trigger(AbilityStartEvent {
                ability_entity: entity,
            });
        }
    }
}

fn remove_base_skill(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    ability_query: Query<Entity, With<Ability>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        info!("just_pressed: remove_base_skill");
        for ability_entity in ability_query.iter() {
            commands.trigger(AbilityRemoveEvent { ability_entity });
        }
    }
}
