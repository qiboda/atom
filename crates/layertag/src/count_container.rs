use std::{fmt::Debug, ops::Deref};

use bevy::{
    prelude::{Component, ReflectComponent},
    reflect::Reflect,
};

use crate::{
    container_op::{LayerTagContainer, LayerTagContainerCondition, LayerTagContainerOp},
    count_layertag::CountLayerTag,
    layertag::LayerTag,
};

#[derive(Debug, Clone, Component, Default, Reflect)]
#[reflect(Component)]
pub struct CountLayerTagContainer {
    layertags: Vec<CountLayerTag>,
}

impl LayerTagContainer for CountLayerTagContainer {
    fn iter_layertag(&self) -> impl Iterator<Item = &LayerTag> {
        self.layertags.iter().map(|x| x.layertag())
    }

    fn exist_layertag(&self, tag: &LayerTag) -> bool {
        self.layertags.iter().any(|x| x.exact_match(tag))
    }

    fn add_layertags(&mut self, layertags: impl Iterator<Item = LayerTag>) {
        for layertag in layertags {
            self.add_layertag(layertag.clone());
        }
    }

    fn remove_layertags<'a>(&mut self, layertags: impl Iterator<Item = &'a LayerTag>) {
        for layertag in layertags {
            self.remove_layertag(layertag);
        }
    }

    fn add_layertag(&mut self, layertag: LayerTag) {
        if let Some(existed_tag) = self.layertags.iter_mut().find(|x| x.exact_match(&layertag)) {
            existed_tag.increase_count();
        } else {
            let mut tag = CountLayerTag::new(layertag);
            tag.reset_count();
            tag.increase_count();
            self.layertags.push(tag);
        }
    }

    fn remove_layertag(&mut self, layertag: &LayerTag) {
        self.layertags.retain_mut(|x| {
            if x.deref().exact_match(layertag) {
                x.decrease_count();
                return x.count() > 0;
            }
            true
        });
    }
}

impl CountLayerTagContainer {
    pub fn get_layertag(&self, layertag: &LayerTag) -> Option<&CountLayerTag> {
        self.layertags.iter().find(|x| x.exact_match(layertag))
    }
}

impl CountLayerTagContainer {
    pub fn receive_op(&mut self, op: impl LayerTagContainerOp, apply: &CountLayerTagContainer) {
        op.operate(self, apply);
    }

    pub fn condition(
        &self,
        condition: impl LayerTagContainerCondition,
        rhs: &CountLayerTagContainer,
    ) -> bool {
        condition.condition(self, rhs)
    }
}

#[cfg(test)]
mod tests {
    extern crate self as layertag;

    use crate::{container_op::LayerTagContainer, layertag::LayerTag, tag::Tag};

    use super::CountLayerTagContainer;

    #[test]
    fn test_layer_counter() {
        let mut layer_tag_container = CountLayerTagContainer::default();

        let tag_a = LayerTag::new(vec![Tag::new("a"), Tag::new("b")]);
        assert_eq!(layer_tag_container.layertags.len(), 0);

        layer_tag_container.add_layertag(tag_a.clone());
        assert_eq!(layer_tag_container.layertags.len(), 1);
        assert_eq!(layer_tag_container.get_layertag(&tag_a).unwrap().count(), 1);

        layer_tag_container.add_layertag(tag_a.clone());
        assert_eq!(layer_tag_container.layertags.len(), 1);
        assert_eq!(layer_tag_container.get_layertag(&tag_a).unwrap().count(), 2);

        layer_tag_container.remove_layertag(&tag_a);
        assert_eq!(layer_tag_container.layertags.len(), 1);
        assert_eq!(layer_tag_container.get_layertag(&tag_a).unwrap().count(), 1);

        layer_tag_container.remove_layertag(&tag_a);
        assert_eq!(layer_tag_container.layertags.len(), 0);
    }
}
