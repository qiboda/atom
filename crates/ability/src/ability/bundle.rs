use bevy::{ecs::system::EntityCommands, prelude::*};
use datatables::effect::TbAbilityRow;

use crate::{
    attribute::attribute_set::AttributeSet,
    bundle::{AbilityBundleTrait, BundleTrait, ReflectAbilityBundleTrait},
    graph::EffectGraphOwner,
    stateset::{StateLayerTagContainer, StateLayerTagRegistry},
};

use super::{
    comp::{Ability, AbilityExecuteState, AbilityTickState},
    layertag::bundle::{AbilityAbortTagBundle, AbilityStartTagBundle},
};

#[derive(Bundle, Default)]
pub struct AbilityOwnerBundle<T: AttributeSet> {
    pub attribute_set: T,
    pub state_set: StateLayerTagContainer,
}

#[derive(Bundle, Reflect, Default)]
#[reflect(AbilityBundleTrait)]
pub struct AbilityBundle {
    pub execute_state: AbilityExecuteState,
    pub tick_state: AbilityTickState,
    pub ability: Ability,
    pub ability_row: TbAbilityRow,
    pub effect_graph_owner: EffectGraphOwner,
    pub start_tag_bundle: AbilityStartTagBundle,
    pub abort_tag_bundle: AbilityAbortTagBundle,
}

impl AbilityBundle {
    pub fn new(ability_row: TbAbilityRow, state_registry: &Res<StateLayerTagRegistry>) -> Self {
        let data = ability_row.data();
        let start_tag_bundle = AbilityStartTagBundle::new(
            &data.start_required_layertags,
            &data.start_disabled_layertags,
            &data.start_added_layertags,
            &data.start_removed_layertags,
            state_registry,
        );

        let abort_tag_bundle = AbilityAbortTagBundle::new(
            &data.abort_required_layertags,
            &data.abort_disabled_layertags,
            state_registry,
        );

        Self {
            ability_row,
            start_tag_bundle,
            abort_tag_bundle,
            ..Default::default()
        }
    }
}

impl BundleTrait for AbilityBundle {
    fn spawn_bundle<'a>(self, commands: &'a mut Commands) -> EntityCommands<'a> {
        commands.spawn(self)
    }
}

impl AbilityBundleTrait for AbilityBundle {}
