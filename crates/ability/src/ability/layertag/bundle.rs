use bevy::{
    log::warn,
    prelude::{Bundle, Res},
    reflect::Reflect,
};
use datatables::effect::RevertableLayerTag;
use layertag::container_op::LayerTagContainer;

use crate::stateset::StateLayerTagRegistry;

use super::tag::{
    AbilityAbortDisableLayerTagContainer, AbilityAbortRequiredLayerTagContainer,
    AbilityAddedLayerTagContainer, AbilityRemovedLayerTagContainer,
    AbilityStartDisableLayerTagContainer, AbilityStartRequiredLayerTagContainer,
};

#[derive(Debug, Default, Bundle, Reflect)]
pub struct AbilityStartTagBundle {
    pub required_layertags: AbilityStartRequiredLayerTagContainer,
    pub disable_layertags: AbilityStartDisableLayerTagContainer,
    pub added_layertags: AbilityAddedLayerTagContainer,
    pub removed_layertags: AbilityRemovedLayerTagContainer,
}

impl AbilityStartTagBundle {
    pub fn new(
        required_layertags: &[String],
        disable_layertags: &[String],
        added_layertags: &[RevertableLayerTag],
        removed_layertags: &[RevertableLayerTag],
        state_registry: &Res<StateLayerTagRegistry>,
    ) -> Self {
        let mut bundle = AbilityStartTagBundle::default();

        for raw_layertag in required_layertags.iter() {
            match state_registry.0.request_from_raw(raw_layertag) {
                Some(layertag) => {
                    bundle.required_layertags.0.add_layertag(layertag);
                }
                None => {
                    warn!("layertag not found registry: {}", raw_layertag)
                }
            }
        }

        for raw_layertag in disable_layertags.iter() {
            match state_registry.0.request_from_raw(raw_layertag) {
                Some(layertag) => {
                    bundle.disable_layertags.0.add_layertag(layertag);
                }
                None => {
                    warn!("layertag not found registry: {}", raw_layertag)
                }
            }
        }

        for revertable_layertag in added_layertags.iter() {
            match state_registry
                .0
                .request_from_raw(&revertable_layertag.raw_layertag)
            {
                Some(layertag) => {
                    bundle
                        .added_layertags
                        .layer_tag_container
                        .add_layertag(layertag);
                    bundle.added_layertags.revert = revertable_layertag.revertable.into();
                }
                None => {
                    warn!("layertag not found registry: {:?}", revertable_layertag)
                }
            }
        }

        for revertable_layertag in removed_layertags.iter() {
            match state_registry
                .0
                .request_from_raw(revertable_layertag.raw_layertag.as_str())
            {
                Some(layertag) => {
                    bundle
                        .removed_layertags
                        .layer_tag_container
                        .add_layertag(layertag);
                    bundle.removed_layertags.revert = revertable_layertag.revertable.into();
                }
                None => {
                    warn!("layertag not found registry: {:?}", revertable_layertag)
                }
            }
        }

        bundle
    }
}

#[derive(Debug, Default, Bundle, Reflect)]
pub struct AbilityAbortTagBundle {
    pub required_layer_tag: AbilityAbortRequiredLayerTagContainer,
    pub disable_layer_tag: AbilityAbortDisableLayerTagContainer,
}

impl AbilityAbortTagBundle {
    pub fn new(
        required_layertags: &[String],
        disable_layertags: &[String],
        state_registry: &Res<StateLayerTagRegistry>,
    ) -> Self {
        let mut bundle = AbilityAbortTagBundle::default();

        for raw_layertag in required_layertags.iter() {
            match state_registry.0.request_from_raw(raw_layertag) {
                Some(layertag) => {
                    bundle.required_layer_tag.0.add_layertag(layertag);
                }
                None => {
                    warn!("layertag not found registry: {}", raw_layertag)
                }
            }
        }

        for raw_layertag in disable_layertags.iter() {
            match state_registry.0.request_from_raw(raw_layertag) {
                Some(layertag) => {
                    bundle.disable_layer_tag.0.add_layertag(layertag);
                }
                None => {
                    warn!("layertag not found registry: {}", raw_layertag)
                }
            }
        }

        bundle
    }
}
