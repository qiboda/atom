use std::ops::Not;

use crate::layertag::LayerTag;

/// 图层标签容器的抽象接口。
pub trait LayerTagContainer {
    /// 迭代容器内所有 [`LayerTag`] 的引用。
    fn iter_layertag(&self) -> impl Iterator<Item = &LayerTag>;

    /// 容器中是否存在与 `tag` 完全匹配的图层标签。
    fn exist_layertag(&self, tag: &LayerTag) -> bool;

    /// 添加一个图层标签。
    fn add_layertag(&mut self, layertag: LayerTag);

    /// 批量添加图层标签。
    fn add_layertags(&mut self, layertag: impl Iterator<Item = LayerTag>);

    /// 移除一个图层标签。
    fn remove_layertag(&mut self, layertag: &LayerTag);

    /// 批量移除图层标签。
    fn remove_layertags<'a>(&mut self, layertag: impl Iterator<Item = &'a LayerTag>);
}

/// 对图层标签容器执行的操作。
pub trait LayerTagContainerOp {
    /// 将本操作应用到 `container`，操作数据来源为 `apply`。
    fn operate(&self, container: &mut impl LayerTagContainer, apply: &impl LayerTagContainer);
}

/// 添加操作：将 `apply` 中的图层标签全部添加到容器。
pub struct LayerTagContainerOpAdd;

impl LayerTagContainerOp for LayerTagContainerOpAdd {
    fn operate(&self, container: &mut impl LayerTagContainer, apply: &impl LayerTagContainer) {
        // TODO: check apply is valid or not?
        container.add_layertags(apply.iter_layertag().cloned())
    }
}

/// 移除操作：将 `apply` 中的图层标签从容器中移除。
pub struct LayerTagContainerOpRemove;

impl LayerTagContainerOp for LayerTagContainerOpRemove {
    fn operate(&self, container: &mut impl LayerTagContainer, apply: &impl LayerTagContainer) {
        container.remove_layertags(apply.iter_layertag());
    }
}

/// 图层标签容器之间的条件判断。
pub trait LayerTagContainerCondition {
    /// 判断 `lhs` 容器是否满足相对 `rhs` 容器的条件。
    fn condition(&self, lhs: &impl LayerTagContainer, rhs: &impl LayerTagContainer) -> bool;
}

/// 必需条件：`container` 必须包含 `required` 中的所有图层标签。
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

/// 排除条件：`container` 必须不包含 `without` 中的任一图层标签。
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
