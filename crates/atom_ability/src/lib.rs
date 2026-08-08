//! # atom_ability
//!
//! 技能系统 crate：基于 Effect Graph（技能图）的技能/增益（Buff）/状态层（StateLayer）实现。
//!
//! 核心概念：
//! - [`graph`]：Effect Graph——用节点（node）与引脚（pin）组成的技能执行图，
//!   节点通过 Blackboard 共享数据，由 executor 驱动执行。
//! - [`ability`]：技能——持有 Effect Graph 实例与状态层标签，作为图执行的入口。
//! - [`buff`]：增益——可叠加的临时状态，带有计时与层数逻辑。
//! - [`effect`]：效果——技能/增益的图实例实体，状态机 + 状态层标签 + 计时。
//! - [`attribute`]：属性——`AttributeSet` 聚合基础属性与修饰符。
//! - [`stateset`]：状态层——技能/增益归属的状态分层。
//!
//! 入口为 [`AbilitySubsystemPlugin`]，它聚合了本 crate 全部子系统插件。

#![deny(missing_docs)]

use ability::{node::ability_entry::EffectNodeAbilityEntryPlugin, plugin::AbilityPlugin};
use bevy::app::{First, Plugin};
use buff::plugin::BuffPlugin;
use effect::EffectPlugin;
use graph::{EffectGraphPlugin, node::plugin::EffectNodePlugin};
use stateset::{StateLayerTagRegistry, init_state_layertag_registry};

pub mod ability;
pub mod attribute;
pub mod buff;
pub mod bundle;
pub mod effect;
pub mod graph;
pub mod stateset;

// TODO: add logs, buff layer.
/// todo: 如果Effect graph中有一个节点是buff，那么这个buff的生命周期会和Effect graph的生命周期一致。
/// 这会判定为技能始终处于激活状态，是不正确的。还是需要添加一个finish的节点。添加finish，会导致State的后续逻辑不执行，是错误的。
/// 不再添加finish node，而是对每一个这种持续存在的节点，增加一个是否Detach的标记。如果detach了，就提前标记为不激活状态。
/// 或者判断后续节点是否有连接，来决定是否设置为不激活状态。
/// 技能子系统聚合插件。
///
/// 一次性注册本 crate 的全部能力：`AbilityPlugin`、`BuffPlugin`、`EffectGraphPlugin`、
/// `EffectNodePlugin`、`EffectNodeAbilityEntryPlugin`、`EffectPlugin`，并初始化状态层标签注册表
/// （[`StateLayerTagRegistry`] + [`init_state_layertag_registry`]）。
#[derive(Debug)]
pub struct AbilitySubsystemPlugin;

impl Plugin for AbilitySubsystemPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(AbilityPlugin)
            .add_plugins(BuffPlugin)
            .add_plugins(EffectGraphPlugin)
            .add_plugins(EffectNodePlugin)
            .add_plugins(EffectNodeAbilityEntryPlugin)
            .add_plugins(EffectPlugin)
            .init_resource::<StateLayerTagRegistry>()
            .add_systems(First, init_state_layertag_registry);
    }
}
