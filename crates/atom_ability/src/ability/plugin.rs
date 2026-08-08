//! 技能插件：注册技能状态更新系统与技能事件 observer。

use bevy::prelude::*;

use crate::graph::{EffectGraphUpdateSystems, state::update_to_despawn_effect_graph};

use super::{
    comp::{Ability, AbilityExecuteState, update_ability_state, update_ability_tick_state},
    event::{
        trigger_ability_abort, trigger_ability_add, trigger_ability_ready, trigger_ability_remove,
        trigger_ability_start, trigger_ability_tickable,
    },
};

/// 技能子系统插件：注册技能状态更新系统与全部技能事件 observer。
#[derive(Debug, Default)]
pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_event::<AbilityStartEvent>()
            //     .add_event::<AbilityReadyEvent>()
            //     .add_event::<AbilityAbortEvent>()
            //     .add_event::<AbilityRemoveEvent>()
            //     .add_event::<AbilityTickableEvent>()
            .add_systems(
                Update,
                (update_ability_state, update_ability_tick_state)
                    .after(EffectGraphUpdateSystems::UpdateState),
            )
            .add_systems(
                Last,
                update_to_despawn_ability.after(update_to_despawn_effect_graph),
            )
            .add_observer(trigger_ability_add)
            .add_observer(trigger_ability_tickable)
            .add_observer(trigger_ability_ready)
            .add_observer(trigger_ability_start)
            .add_observer(trigger_ability_remove)
            .add_observer(trigger_ability_abort);
    }
}

/// 销毁待移除的技能：`ToRemove` 且子图已全部清除时 despawn 技能实体。
pub fn update_to_despawn_ability(
    mut commands: Commands,
    query: Query<(Entity, &AbilityExecuteState, &Children), With<Ability>>,
) {
    for (entity, state, children) in query.iter() {
        if *state == AbilityExecuteState::ToRemove && children.is_empty() {
            commands.entity(entity).despawn();
        }
    }
}
