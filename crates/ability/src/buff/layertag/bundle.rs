use bevy::{
    log::warn,
    prelude::{Bundle, Res},
    reflect::Reflect,
};
use datatables::effect::RevertableLayerTag;
use layertag::container_op::LayerTagContainer;

use crate::stateset::StateLayerTagRegistry;

use super::tag::{
    BuffAbortDisableLayerTagContainer, BuffAbortRequiredLayerTagContainer,
    BuffAddedLayerTagContainer, BuffRemovedLayerTagContainer, BuffStartDisableLayerTagContainer,
    BuffStartRequiredLayerTagContainer,
};

#[derive(Debug, Default, Bundle, Reflect)]
pub struct BuffStartTagBundle {
    pub required_layertags: BuffStartRequiredLayerTagContainer,
    pub disable_layertags: BuffStartDisableLayerTagContainer,
    pub added_layertags: BuffAddedLayerTagContainer,
    pub removed_layertags: BuffRemovedLayerTagContainer,
}

impl BuffStartTagBundle {
    pub fn new(
        required_layertags: &[String],
        disable_layertags: &[String],
        added_layertags: &[RevertableLayerTag],
        removed_layertags: &[RevertableLayerTag],
        state_registry: &Res<StateLayerTagRegistry>,
    ) -> Self {
        let mut bundle = BuffStartTagBundle::default();

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
pub struct BuffAbortTagBundle {
    pub required_layer_tag: BuffAbortRequiredLayerTagContainer,
    pub disable_layer_tag: BuffAbortDisableLayerTagContainer,
}

impl BuffAbortTagBundle {
    pub fn new(
        required_layertags: &[String],
        disable_layertags: &[String],
        state_registry: &Res<StateLayerTagRegistry>,
    ) -> Self {
        let mut bundle = BuffAbortTagBundle::default();

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
