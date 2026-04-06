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
