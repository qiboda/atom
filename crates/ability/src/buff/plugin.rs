use crate::{
    buff::{
        event::{
            trigger_buff_abort, trigger_buff_on_add, trigger_buff_remove, trigger_buff_start,
            trigger_buff_tickable, BuffAbortEvent, BuffReadyEvent, BuffRemoveEvent, BuffStartEvent,
            BuffTickableEvent,
        },
        state::{update_buff_state, update_buff_tick_state, Buff, BuffExecuteState},
        timer::update_buff_time_system,
    },
    graph::{state::update_to_despawn_effect_graph, EffectGraphUpdateSystemSet},
};
use bevy::prelude::*;

#[derive(Debug, Default)]
pub struct BuffPlugin;

impl Plugin for BuffPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                BuffUpdateSystemSet::UpdateTime,
                BuffUpdateSystemSet::UpdateState,
            )
                .chain(),
        )
        .add_event::<BuffReadyEvent>()
        .add_event::<BuffStartEvent>()
        .add_event::<BuffRemoveEvent>()
        .add_event::<BuffAbortEvent>()
        .add_event::<BuffTickableEvent>()
        .observe(trigger_buff_on_add)
        .observe(trigger_buff_remove)
        .observe(trigger_buff_start)
        .observe(trigger_buff_abort)
        .observe(trigger_buff_tickable)
        .add_systems(
            Update,
            (update_buff_state, update_buff_tick_state)
                .after(EffectGraphUpdateSystemSet::UpdateState)
                .in_set(BuffUpdateSystemSet::UpdateState),
        )
        .add_systems(
            Update,
            update_buff_time_system.in_set(BuffUpdateSystemSet::UpdateTime),
        )
        .add_systems(
            Last,
            update_to_despawn_buff.after(update_to_despawn_effect_graph),
        );
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum BuffUpdateSystemSet {
    UpdateState,
    UpdateTime,
}

pub fn update_to_despawn_buff(
    mut commands: Commands,
    query: Query<(Entity, &BuffExecuteState, &Children), With<Buff>>,
) {
    for (entity, state, children) in query.iter() {
        if *state == BuffExecuteState::ToRemove && children.len() == 0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
