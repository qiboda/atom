use std::{fmt::Debug, ops::ControlFlow};

use bevy::reflect::{reflect_trait, Reflect};

use crate::tag::Tag;

/// A tag that can be used to identify a layer, and with struct data.
#[reflect_trait]
pub trait LayerTag: Reflect + Debug {
    fn tag(&self) -> &[Tag];

    /// two tag exact match
    /// for example:
    /// ```ignore
    /// "a.b.c" == "a.b.c"
    /// "a.b" != "a.b.c"
    /// ```
    fn exact_match(&self, rhs: &dyn LayerTag) -> bool {
        self.tag() == rhs.tag()
    }

    /// two tag exact match
    /// for example
    /// ```ignore
    /// "a.b.c" == "a.b.c"
    /// "a.b" == "a.b.c"
    /// "a.b.d" != "a.b.c.d"
    /// ```
    fn partical_match(&self, rhs: &dyn LayerTag) -> bool {
        let r = self
            .tag()
            .iter()
            .zip(rhs.tag().iter())
            .try_for_each(|(x, y)| {
                if x == y {
                    return ControlFlow::Continue(());
                }
                ControlFlow::Break(())
            });

        r == ControlFlow::Continue(())
    }

    /// get same prefix
    /// for example
    /// ```ignore
    /// "a.b.c" -> "a.b.d" -> "a.b"
    /// "a.b.c" -> "a.b.c" -> "a.b.c"
    /// "" -> "a.b.c" -> ""
    /// ```
    fn same_prefix(&self, rhs: &dyn LayerTag) -> Vec<&Tag> {
        let mut ret = vec![];
        self.tag()
            .iter()
            .zip(rhs.tag().iter())
            .try_for_each(|(x, y)| {
                if x == y {
                    ret.push(x);
                    return ControlFlow::Continue(());
                }
                ControlFlow::Break(())
            });
        ret
    }
}

#[cfg(test)]
mod test {
    use core::fmt;

    use crate::{layertag::LayerTag, tag::Tag};
    use bevy::reflect::Reflect;

    use once_cell::sync::OnceCell;

    #[derive(Reflect, Debug, Clone)]
    struct TestTag {}

    impl LayerTag for TestTag {
        fn tag(&self) -> &[Tag] {
            static CELL: OnceCell<Vec<Tag>> = OnceCell::new();
            CELL.get_or_init(|| vec![Tag::new("a"), Tag::new("b"), Tag::new("c")])
                .as_slice()
        }
    }

    impl fmt::Display for TestTag {
        fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt::Result::Ok(())
        }
    }

    #[derive(Reflect, Debug, Clone)]
    struct TestTag2 {}

    impl LayerTag for TestTag2 {
        fn tag(&self) -> &[Tag] {
            static CELL: OnceCell<Vec<Tag>> = OnceCell::new();
            CELL.get_or_init(|| vec![Tag::new("a"), Tag::new("b"), Tag::new("d")])
                .as_slice()
        }
    }

    #[derive(Reflect, Debug, Clone)]
    struct TestTag3 {}

    impl LayerTag for TestTag3 {
        fn tag(&self) -> &[Tag] {
            static CELL: OnceCell<Vec<Tag>> = OnceCell::new();
            CELL.get_or_init(|| vec![Tag::new("a"), Tag::new("b")])
                .as_slice()
        }
    }

    #[allow(clippy::bool_assert_comparison)]
    #[test]
    fn cmp_layer_tag() {
        let test_tag = TestTag {};
        let test_tag_dup = TestTag {};
        let test_tag_2 = TestTag2 {};
        let test_tag_3 = TestTag3 {};

        assert_eq!(test_tag_dup.exact_match(&test_tag), true);
        assert_eq!(test_tag.exact_match(&test_tag_2), false);

        assert_eq!(test_tag.partical_match(&test_tag_2), false);
        assert_eq!(test_tag.partical_match(&test_tag_3), true);

        assert_eq!(
            test_tag.same_prefix(&test_tag_2),
            vec![&Tag::new("a"), &Tag::new("b")]
        );

        assert_eq!(
            test_tag.same_prefix(&test_tag_3),
            vec![&Tag::new("a"), &Tag::new("b")]
        );
    }
}
