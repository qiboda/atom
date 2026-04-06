use crate::{
    buff::{
        event::{
            trigger_buff_abort, trigger_buff_on_add, trigger_buff_remove, trigger_buff_start,
            trigger_buff_tickable,
        },
        state::{Buff, BuffExecuteState, update_buff_state, update_buff_tick_state},
        timer::update_buff_time_system,
    },
    graph::{EffectGraphUpdateSystems, state::update_to_despawn_effect_graph},
};
use bevy::prelude::*;

#[derive(Debug, Default)]
pub struct BuffPlugin;

impl Plugin for BuffPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                BuffUpdateSystems::UpdateTime,
                BuffUpdateSystems::UpdateState,
            )
                .chain(),
        )
        // .add_event::<BuffReadyEvent>()
        // .add_event::<BuffStartEvent>()
        // .add_event::<BuffRemoveEvent>()
        // .add_event::<BuffAbortEvent>()
        // .add_event::<BuffTickableEvent>()
        .add_observer(trigger_buff_on_add)
        .add_observer(trigger_buff_remove)
        .add_observer(trigger_buff_start)
        .add_observer(trigger_buff_abort)
        .add_observer(trigger_buff_tickable)
        .add_systems(
            Update,
            (update_buff_state, update_buff_tick_state)
                .after(EffectGraphUpdateSystems::UpdateState)
                .in_set(BuffUpdateSystems::UpdateState),
        )
        .add_systems(
            Update,
            update_buff_time_system.in_set(BuffUpdateSystems::UpdateTime),
        )
        .add_systems(
            Last,
            update_to_despawn_buff.after(update_to_despawn_effect_graph),
        );
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum BuffUpdateSystems {
    UpdateState,
    UpdateTime,
}

pub fn update_to_despawn_buff(
    mut commands: Commands,
    query: Query<(Entity, &BuffExecuteState, &Children), With<Buff>>,
) {
    for (entity, state, children) in query.iter() {
        if *state == BuffExecuteState::ToRemove && children.is_empty() {
            commands.entity(entity).despawn();
        }
    }
}
