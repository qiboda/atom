use bevy::prelude::*;

use crate::graph::{state::update_to_despawn_effect_graph, EffectGraphUpdateSystemSet};

use super::{
    comp::{update_ability_state, update_ability_tick_state, Ability, AbilityExecuteState},
    event::{
        trigger_ability_abort, trigger_ability_add, trigger_ability_ready, trigger_ability_remove,
        trigger_ability_start, trigger_ability_tickable, AbilityAbortEvent, AbilityReadyEvent,
        AbilityRemoveEvent, AbilityStartEvent, AbilityTickableEvent,
    },
};

#[derive(Debug, Default)]
pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AbilityStartEvent>()
            .add_event::<AbilityReadyEvent>()
            .add_event::<AbilityAbortEvent>()
            .add_event::<AbilityRemoveEvent>()
            .add_event::<AbilityTickableEvent>()
            .add_systems(
                Update,
                (update_ability_state, update_ability_tick_state)
                    .after(EffectGraphUpdateSystemSet::UpdateState),
            )
            .add_systems(
                Last,
                update_to_despawn_ability.after(update_to_despawn_effect_graph),
            )
            .observe(trigger_ability_add)
            .observe(trigger_ability_tickable)
            .observe(trigger_ability_ready)
            .observe(trigger_ability_start)
            .observe(trigger_ability_remove)
            .observe(trigger_ability_abort);
    }
}

pub fn update_to_despawn_ability(
    mut commands: Commands,
    query: Query<(Entity, &AbilityExecuteState, &Children), With<Ability>>,
) {
    for (entity, state, children) in query.iter() {
        if *state == AbilityExecuteState::ToRemove && children.len() == 0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
