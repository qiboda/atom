use std::fmt::Debug;

use bevy::{
    platform::collections::HashSet,
    prelude::{Component, ReflectComponent},
    reflect::Reflect,
};

use crate::{
    container_op::{LayerTagContainer, LayerTagContainerCondition, LayerTagContainerOp},
    layertag::LayerTag,
};

/// 去重的图层标签容器：以 `HashSet` 存储，同一 [`LayerTag`] 只保留一份。
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
    /// 以 `apply` 容器为操作来源，对自身执行 `op` 操作。
    pub fn receive_op(&mut self, op: impl LayerTagContainerOp, apply: &SingleLayerTagContainer) {
        op.operate(self, apply);
    }

    /// 以 `rhs` 容器为参照，对自身执行 `condition` 条件判断。
    pub fn condition(
        &self,
        condition: impl LayerTagContainerCondition,
        rhs: &SingleLayerTagContainer,
    ) -> bool {
        condition.condition(self, rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::SingleLayerTagContainer;
    use crate::{container_op::LayerTagContainer, layertag::LayerTag, tag::Tag};

    #[test]
    fn test_add_and_exist() {
        let mut container = SingleLayerTagContainer::default();
        let tag = LayerTag::new(vec![Tag::new("a"), Tag::new("b")]);
        assert!(!container.exist_layertag(&tag));
        container.add_layertag(tag.clone());
        assert!(container.exist_layertag(&tag));
    }

    #[test]
    fn test_add_idempotent() {
        let mut container = SingleLayerTagContainer::default();
        let tag = LayerTag::new(vec![Tag::new("x")]);
        container.add_layertag(tag.clone());
        container.add_layertag(tag.clone());
        assert!(container.exist_layertag(&tag));
        // Count via iter
        assert_eq!(container.iter_layertag().count(), 1);
    }

    #[test]
    fn test_remove() {
        let mut container = SingleLayerTagContainer::default();
        let tag = LayerTag::new(vec![Tag::new("a")]);
        container.add_layertag(tag.clone());
        container.remove_layertag(&tag);
        assert!(!container.exist_layertag(&tag));
    }

    #[test]
    fn test_multiple_tags() {
        let mut container = SingleLayerTagContainer::default();
        let tag1 = LayerTag::new(vec![Tag::new("a")]);
        let tag2 = LayerTag::new(vec![Tag::new("b")]);
        container.add_layertag(tag1.clone());
        container.add_layertag(tag2.clone());
        assert_eq!(container.iter_layertag().count(), 2);
        container.remove_layertag(&tag1);
        assert_eq!(container.iter_layertag().count(), 1);
        assert!(container.exist_layertag(&tag2));
    }
}
