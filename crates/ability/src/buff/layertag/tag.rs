use bevy::prelude::*;
use layertag::count_container::CountLayerTagContainer;

#[derive(Component, Debug, Default, Reflect)]
pub struct BuffStartRequiredLayerTagContainer(pub CountLayerTagContainer);

#[derive(Component, Debug, Default, Reflect)]
pub struct BuffStartDisableLayerTagContainer(pub CountLayerTagContainer);

#[derive(Component, Debug, Default, Reflect)]
pub struct BuffAbortRequiredLayerTagContainer(pub CountLayerTagContainer);

#[derive(Component, Debug, Default, Reflect)]
pub struct BuffAbortDisableLayerTagContainer(pub CountLayerTagContainer);

#[derive(Debug, Default, Reflect, PartialEq, Eq)]
pub enum BuffLayerTagContainerRevert {
    #[default]
    No,
    Yes,
}

impl From<bool> for BuffLayerTagContainerRevert {
    fn from(value: bool) -> Self {
        if value {
            BuffLayerTagContainerRevert::Yes
        } else {
            BuffLayerTagContainerRevert::No
        }
    }
}

#[derive(Component, Debug, Default, Reflect)]
pub struct BuffAddedLayerTagContainer {
    pub layer_tag_container: CountLayerTagContainer,
    pub revert: BuffLayerTagContainerRevert,
}

#[derive(Component, Debug, Default, Reflect)]
pub struct BuffRemovedLayerTagContainer {
    pub layer_tag_container: CountLayerTagContainer,
    pub revert: BuffLayerTagContainerRevert,
}
