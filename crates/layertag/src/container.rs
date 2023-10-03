use std::{
    fmt::Debug,
    ops::{Deref, Not},
    slice::{Iter, IterMut},
};

use crate::layertag::LayerTag;
use bevy::{prelude::Component, reflect::Reflect};

#[derive(Debug, Component, Default, Reflect)]
pub struct LayerTagContainer {
    tags: Vec<Box<dyn LayerTag>>,
}

impl Clone for LayerTagContainer {
    fn clone(&self) -> Self {
        Self {
            tags: self.tags.iter().map(|x| x.box_clone()).collect(),
        }
    }
}

impl LayerTagContainer {
    pub fn iter(&self) -> Iter<'_, Box<dyn LayerTag>> {
        self.tags.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, Box<dyn LayerTag>> {
        self.tags.iter_mut()
    }
}

impl LayerTagContainer {
    pub fn add_layer_tag(&mut self, tag: &dyn LayerTag) {
        assert!(self
            .iter()
            .any(|x| tag.deref().exact_match((*x).deref()))
            .not());
        self.tags.push(tag.box_clone());
    }

    pub fn remove_enable_tag(&mut self, tag: &dyn LayerTag) {
        self.tags
            .retain(|x| x.deref().exact_match(tag.deref()).not());
    }
}

impl LayerTagContainer {
    pub fn receive_op(&mut self, op: impl LayerTagContainerOp, apply: &LayerTagContainer) {
        op.operate(self, apply);
    }

    pub fn condition(
        &self,
        condition: impl LayerTagContainerCondition,
        rhs: &LayerTagContainer,
    ) -> bool {
        condition.condition(self, rhs)
    }
}

pub trait LayerTagContainerOp {
    /// operate apply to container.
    fn operate(&self, container: &mut LayerTagContainer, apply: &LayerTagContainer);
}

pub struct LayerTagContainerOpAdd;

impl LayerTagContainerOp for LayerTagContainerOpAdd {
    fn operate(&self, container: &mut LayerTagContainer, apply: &LayerTagContainer) {
        // todo: check apply is valid or not?
        apply.iter().for_each(|x| {
            container.add_layer_tag(x.deref());
        });
    }
}

pub struct LayerTagContainerOpRemove;

impl LayerTagContainerOp for LayerTagContainerOpRemove {
    fn operate(&self, container: &mut LayerTagContainer, apply: &LayerTagContainer) {
        apply.iter().for_each(|x| {
            container.remove_enable_tag(x.deref());
        });
    }
}

pub trait LayerTagContainerCondition {
    fn condition(&self, lhs: &LayerTagContainer, rhs: &LayerTagContainer) -> bool;
}

pub struct LayerTagContainerConditionRequired;

impl LayerTagContainerCondition for LayerTagContainerConditionRequired {
    fn condition(&self, container: &LayerTagContainer, required: &LayerTagContainer) -> bool {
        required
            .iter()
            .all(|x| container.iter().any(|y| x.deref().exact_match(y.deref())))
    }
}

pub struct LayerTagContainerConditionWithout;

impl LayerTagContainerCondition for LayerTagContainerConditionWithout {
    fn condition(&self, container: &LayerTagContainer, without: &LayerTagContainer) -> bool {
        without.iter().all(|x| {
            container
                .iter()
                .any(|y| x.deref().exact_match(y.deref()))
                .not()
        })
    }
}
