use bevy::{prelude::Bundle, reflect::Reflect};
use layertag::container::LayerTagContainer;

use crate::{ability::AbilityBase, attribute::attribute_set::AttributeSet};

/// ability owner entity
///     ability enitty 1
///     ability base
///        &ability graph
///     ability tag container
///     other
///
///     ability enitty 2
///     ability base
///        &ability graph
///     ability tag container
///     other
///
/// attribute set
/// state set

#[derive(Bundle, Default)]
pub struct AbilitySubsystemBundle<T: AttributeSet> {
    pub attribute_set: T,
    pub state_set: LayerTagContainer,
}

#[derive(Bundle, Reflect, Clone)]
pub struct AbilityBundle {
    pub ability: AbilityBase,
    pub tag_contaier: LayerTagContainer,
}
