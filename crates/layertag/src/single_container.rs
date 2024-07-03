use std::fmt::Debug;

use bevy::{
    prelude::{Component, ReflectComponent},
    reflect::Reflect,
    utils::HashSet,
};

use crate::{
    container_op::{LayerTagContainer, LayerTagContainerCondition, LayerTagContainerOp},
    layertag::LayerTag,
};

#[derive(Debug, Clone, Component, Default, Reflect)]
#[reflect(Component)]
pub struct SingleLayerTagContainer {
    layertags: HashSet<LayerTag>,
}

impl LayerTagContainer for SingleLayerTagContainer {
    fn iter_layertag(&self) -> impl Iterator<Item = &LayerTag> {
        Box::new(self.layertags.iter())
    }

    fn exist_layertag(&self, tag: &LayerTag) -> bool {
        self.layertags.iter().any(|x| x.exact_match(tag))
    }

    fn add_layertags(&mut self, layertags: impl Iterator<Item = LayerTag>) {
        for layertag in layertags {
            self.layertags.insert(layertag.clone());
        }
    }

    fn remove_layertags<'a>(&mut self, layertags: impl Iterator<Item = &'a LayerTag>) {
        for layertag in layertags {
            self.layertags.remove(layertag);
        }
    }

    fn add_layertag(&mut self, layertag: LayerTag) {
        self.layertags.insert(layertag.clone());
    }

    fn remove_layertag(&mut self, layertag: &LayerTag) {
        self.layertags.remove(layertag);
    }
}

impl SingleLayerTagContainer {
    pub fn receive_op(&mut self, op: impl LayerTagContainerOp, apply: &SingleLayerTagContainer) {
        op.operate(self, apply);
    }

    pub fn condition(
        &self,
        condition: impl LayerTagContainerCondition,
        rhs: &SingleLayerTagContainer,
    ) -> bool {
        condition.condition(self, rhs)
    }
}
