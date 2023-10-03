use std::{
    fmt::Debug,
    ops::{Deref, DerefMut, Not},
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
    pub fn get_layer_tag(&self, tag: &dyn LayerTag) -> Option<&dyn LayerTag> {
        self.tags
            .iter()
            .find(|x| x.exact_match(tag))
            .map(|x| x.deref())
    }

    pub fn get_layer_tag_mut(&mut self, tag: &dyn LayerTag) -> Option<&mut dyn LayerTag> {
        self.tags
            .iter_mut()
            .find(|x| x.exact_match(tag))
            .map(|x| x.deref_mut())
    }
}

impl LayerTagContainer {
    pub fn add_layer_tag(&mut self, tag: &dyn LayerTag) {
        if let Some(existed_tag) = self.tags.iter_mut().find(|x| x.exact_match(tag)) {
            existed_tag.increase_count();
        } else {
            let mut tag = tag.box_clone();
            tag.reset_count();
            tag.increase_count();
            self.tags.push(tag);
        }
    }

    pub fn remove_layer_tag(&mut self, tag: &dyn LayerTag) {
        self.tags.retain_mut(|x| {
            if x.deref().exact_match(tag.deref()) {
                x.decrease_count();
                return x.count() > 0;
            }
            true
        });
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
            container.remove_layer_tag(x.deref());
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

#[cfg(test)]
mod tests {
    extern crate self as layertag;
    use bevy::reflect::Reflect;
    use layertag_derive::LayerTag;

    use super::LayerTagContainer;

    #[derive(LayerTag, Debug, Clone, PartialEq, Eq, Reflect)]
    #[layer_tag()]
    struct TagA {
        #[layer_tag_counter]
        pub value: usize,
    }

    #[test]
    fn test_layer_counter() {
        let mut layer_tag_container = LayerTagContainer::default();

        let tag_a = TagA { value: 10 };
        assert_eq!(layer_tag_container.tags.len(), 0);

        layer_tag_container.add_layer_tag(&tag_a);
        assert_eq!(layer_tag_container.tags.len(), 1);
        assert_eq!(
            layer_tag_container.get_layer_tag(&tag_a).unwrap().count(),
            1
        );

        layer_tag_container.add_layer_tag(&tag_a);
        assert_eq!(layer_tag_container.tags.len(), 1);
        assert_eq!(
            layer_tag_container.get_layer_tag(&tag_a).unwrap().count(),
            2
        );

        layer_tag_container.remove_layer_tag(&tag_a);
        assert_eq!(layer_tag_container.tags.len(), 1);
        assert_eq!(
            layer_tag_container.get_layer_tag(&tag_a).unwrap().count(),
            1
        );

        layer_tag_container.remove_layer_tag(&tag_a);
        assert_eq!(layer_tag_container.tags.len(), 0);
    }
}
