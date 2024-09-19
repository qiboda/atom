use ability::{node::ability_entry::EffectNodeAbilityEntryPlugin, plugin::AbilityPlugin};
use bevy::app::{First, Plugin};
use buff::plugin::BuffPlugin;
use graph::{node::plugin::EffectNodePlugin, EffectGraphPlugin};
use stateset::{init_state_layertag_registry, StateLayerTagRegistry};

pub mod ability;
pub mod attribute;
pub mod buff;
pub mod bundle;
pub mod graph;
pub mod stateset;

// TODO: add logs, buff layer.
/// todo: 如果Effect graph中有一个节点是buff，那么这个buff的生命周期会和Effect graph的生命周期一致。
/// 这会判定为技能始终处于激活状态，是不正确的。还是需要添加一个finish的节点。添加finish，会导致State的后续逻辑不执行，是错误的。
/// 不再添加finish node，而是对每一个这种持续存在的节点，增加一个是否Detach的标记。如果detach了，就提前标记为不激活状态。
/// 或者判断后续节点是否有连接，来决定是否设置为不激活状态。
#[derive(Debug)]
pub struct AbilitySubsystemPlugin;

impl Plugin for AbilitySubsystemPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(AbilityPlugin)
            .add_plugins(BuffPlugin)
            .add_plugins(EffectGraphPlugin)
            .add_plugins(EffectNodePlugin)
            .add_plugins(EffectNodeAbilityEntryPlugin)
            .init_resource::<StateLayerTagRegistry>()
            .add_systems(First, init_state_layertag_registry);
    }
}
