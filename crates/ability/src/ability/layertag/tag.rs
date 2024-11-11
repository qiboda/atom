use bevy::prelude::*;
use layertag::count_container::CountLayerTagContainer;

#[derive(Component, Debug, Default, Reflect)]
pub struct AbilityStartRequiredLayerTagContainer(pub CountLayerTagContainer);

#[derive(Component, Debug, Default, Reflect)]
pub struct AbilityStartDisableLayerTagContainer(pub CountLayerTagContainer);

#[derive(Component, Debug, Default, Reflect)]
pub struct AbilityAbortRequiredLayerTagContainer(pub CountLayerTagContainer);

#[derive(Component, Debug, Default, Reflect)]
pub struct AbilityAbortDisableLayerTagContainer(pub CountLayerTagContainer);

#[derive(Debug, Default, Reflect, PartialEq, Eq)]
pub enum AbilityLayerTagContainerRevert {
    #[default]
    No,
    Yes,
}

impl From<bool> for AbilityLayerTagContainerRevert {
    fn from(value: bool) -> Self {
        if value {
            AbilityLayerTagContainerRevert::Yes
        } else {
            AbilityLayerTagContainerRevert::No
        }
    }
}

#[derive(Component, Debug, Default, Reflect)]
pub struct AbilityAddedLayerTagContainer {
    pub layer_tag_container: CountLayerTagContainer,
    pub revert: AbilityLayerTagContainerRevert,
}

#[derive(Component, Debug, Default, Reflect)]
pub struct AbilityRemovedLayerTagContainer {
    pub layer_tag_container: CountLayerTagContainer,
    pub revert: AbilityLayerTagContainerRevert,
}
