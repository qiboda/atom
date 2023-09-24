use bevy::prelude::Bundle;

use crate::{
    ability::{ability_tag::AbilityTagContainer, AbilityBase},
    attribute::attribute_set::AttributeSet, state::StateTagContainer,
};

/// ability owner entity
///     ability enitty 1
///          ability base
///             &ability graph
///          other
///     ability enitty 2
///          ability base
///             &ability graph
///          other
///     attribute set

#[derive(Bundle, Default)]
pub struct AbilitySubsystemBundle<T: AttributeSet> {
    pub attribute_set: T,
    pub state_set: StateTagContainer
}

#[derive(Bundle)]
pub struct AbilityBundle {
    pub ability: AbilityBase,
    pub tag_contaier: AbilityTagContainer,
}
