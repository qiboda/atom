use std::ops::Not;

use crate::layertag::LayerTag;

pub trait LayerTagContainer {
    fn iter_layertag(&self) -> impl Iterator<Item = &LayerTag>;

    fn exist_layertag(&self, tag: &LayerTag) -> bool;

    fn add_layertag(&mut self, layertag: LayerTag);

    fn add_layertags(&mut self, layertag: impl Iterator<Item = LayerTag>);

    fn remove_layertag(&mut self, layertag: &LayerTag);

    fn remove_layertags<'a>(&mut self, layertag: impl Iterator<Item = &'a LayerTag>);
}

pub trait LayerTagContainerOp {
    /// operate apply to container.
    fn operate(&self, container: &mut impl LayerTagContainer, apply: &impl LayerTagContainer);
}

pub struct LayerTagContainerOpAdd;

impl LayerTagContainerOp for LayerTagContainerOpAdd {
    fn operate(&self, container: &mut impl LayerTagContainer, apply: &impl LayerTagContainer) {
        // TODO: check apply is valid or not?
        container.add_layertags(apply.iter_layertag().cloned())
    }
}

pub struct LayerTagContainerOpRemove;

impl LayerTagContainerOp for LayerTagContainerOpRemove {
    fn operate(&self, container: &mut impl LayerTagContainer, apply: &impl LayerTagContainer) {
        container.remove_layertags(apply.iter_layertag());
    }
}

pub trait LayerTagContainerCondition {
    fn condition(&self, lhs: &impl LayerTagContainer, rhs: &impl LayerTagContainer) -> bool;
}

pub struct LayerTagContainerConditionRequired;

impl LayerTagContainerCondition for LayerTagContainerConditionRequired {
    fn condition(
        &self,
        container: &impl LayerTagContainer,
        required: &impl LayerTagContainer,
    ) -> bool {
        required
            .iter_layertag()
            .all(|x| container.iter_layertag().any(|y| x.exact_match(y)))
    }
}

pub struct LayerTagContainerConditionWithout;

impl LayerTagContainerCondition for LayerTagContainerConditionWithout {
    fn condition(
        &self,
        container: &impl LayerTagContainer,
        without: &impl LayerTagContainer,
    ) -> bool {
        without
            .iter_layertag()
            .all(|x| container.iter_layertag().any(|y| x.exact_match(y)).not())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{single_container::SingleLayerTagContainer, tag::Tag};

    fn make_tag(name: &'static str) -> LayerTag {
        LayerTag::new(vec![Tag::new(name)])
    }

    #[test]
    fn test_op_add() {
        let mut container = SingleLayerTagContainer::default();
        let mut apply = SingleLayerTagContainer::default();
        apply.add_layertag(make_tag("x"));
        apply.add_layertag(make_tag("y"));

        LayerTagContainerOpAdd.operate(&mut container, &apply);
        assert!(container.exist_layertag(&make_tag("x")));
        assert!(container.exist_layertag(&make_tag("y")));
    }

    #[test]
    fn test_op_remove() {
        let mut container = SingleLayerTagContainer::default();
        container.add_layertag(make_tag("a"));
        container.add_layertag(make_tag("b"));

        let mut remove_set = SingleLayerTagContainer::default();
        remove_set.add_layertag(make_tag("a"));

        LayerTagContainerOpRemove.operate(&mut container, &remove_set);
        assert!(!container.exist_layertag(&make_tag("a")));
        assert!(container.exist_layertag(&make_tag("b")));
    }

    #[test]
    fn test_condition_required_satisfied() {
        let mut container = SingleLayerTagContainer::default();
        container.add_layertag(make_tag("a"));
        container.add_layertag(make_tag("b"));

        let mut required = SingleLayerTagContainer::default();
        required.add_layertag(make_tag("a"));

        assert!(LayerTagContainerConditionRequired.condition(&container, &required));
    }

    #[test]
    fn test_condition_required_not_satisfied() {
        let mut container = SingleLayerTagContainer::default();
        container.add_layertag(make_tag("a"));

        let mut required = SingleLayerTagContainer::default();
        required.add_layertag(make_tag("b"));

        assert!(!LayerTagContainerConditionRequired.condition(&container, &required));
    }

    #[test]
    fn test_condition_without_satisfied() {
        let mut container = SingleLayerTagContainer::default();
        container.add_layertag(make_tag("a"));

        let mut without = SingleLayerTagContainer::default();
        without.add_layertag(make_tag("b"));

        assert!(LayerTagContainerConditionWithout.condition(&container, &without));
    }

    #[test]
    fn test_condition_without_not_satisfied() {
        let mut container = SingleLayerTagContainer::default();
        container.add_layertag(make_tag("a"));

        let mut without = SingleLayerTagContainer::default();
        without.add_layertag(make_tag("a"));

        assert!(!LayerTagContainerConditionWithout.condition(&container, &without));
    }
}
