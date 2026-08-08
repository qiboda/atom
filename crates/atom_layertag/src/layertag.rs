use std::{
    borrow::Cow,
    fmt::{self, Debug, Display},
    ops::ControlFlow,
};

use bevy::reflect::Reflect;

use crate::tag::Tag;

/// 图层标签：由多个 [`Tag`] 片段以分隔符（`.`）连接组成的完整路径。
#[derive(Debug, Clone, Reflect, PartialEq, Eq, Hash)]
pub struct LayerTag {
    /// 组成该图层标签的片段序列。
    pub tags: Cow<'static, [Tag]>,
}

impl LayerTag {
    /// 图层标签片段之间的分隔符。
    pub const DELIMITER: &'static str = ".";
}

impl Display for LayerTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw_layertag())
    }
}

impl LayerTag {
    pub(crate) fn new(tags: Vec<Tag>) -> Self {
        Self {
            tags: Cow::Owned(tags),
        }
    }

    pub(crate) fn new_from_raw(raw_tag: &str) -> Self {
        let tags: Vec<Tag> = raw_tag
            .split(LayerTag::DELIMITER)
            .map(|x| Tag::new(x.to_owned()))
            .collect();
        Self {
            tags: Cow::Owned(tags),
        }
    }
}

impl LayerTag {
    /// 返回该图层标签的片段序列。
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }

    /// 返回以分隔符连接后的完整字符串表示。
    pub fn raw_layertag(&self) -> String {
        self.tags
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(LayerTag::DELIMITER)
    }

    /// 精确匹配：两个图层标签的片段序列完全相同。
    ///
    /// 例如：
    /// ```ignore
    /// "a.b.c" == "a.b.c"
    /// "a.b" != "a.b.c"
    /// ```
    pub fn exact_match(&self, rhs: &LayerTag) -> bool {
        self.tags() == rhs.tags()
    }

    /// 部分匹配：较短一方的片段序列是另一方的片段前缀时返回 `true`。
    ///
    /// 例如：
    /// ```ignore
    /// "a.b.c" == "a.b.c"
    /// "a.b" == "a.b.c"
    /// "a.b.d" != "a.b.c.d"
    /// ```
    pub fn partial_match(&self, rhs: &LayerTag) -> bool {
        let r = self
            .tags()
            .iter()
            .zip(rhs.tags().iter())
            .try_for_each(|(x, y)| {
                if x == y {
                    return ControlFlow::Continue(());
                }
                ControlFlow::Break(())
            });

        r == ControlFlow::Continue(())
    }

    /// 取相同位置上的相同片段（两个图层标签的公共前缀）。
    ///
    /// 例如：
    /// ```ignore
    /// "a.b.c" -> "a.b.d" -> "a.b"
    /// "a.b.c" -> "a.b.c" -> "a.b.c"
    /// "" -> "a.b.c" -> ""
    /// ```
    pub fn same_prefix<'a>(&'a self, rhs: &'a LayerTag) -> impl Iterator<Item = &'a Tag> {
        self.tags()
            .iter()
            .zip(rhs.tags().iter())
            .filter_map(|(x, y)| if *x == *y { Some(x) } else { None })
    }
}

#[cfg(test)]
mod test {
    extern crate self as layertag;

    use crate::{layertag::LayerTag, tag::Tag};

    #[allow(clippy::bool_assert_comparison)]
    #[test]
    fn cmp_layer_tag() {
        let test_tag = LayerTag::new(vec![Tag::new("a"), Tag::new("b")]);
        let test_tag_dup = LayerTag::new(vec![Tag::new("a"), Tag::new("b")]);
        let test_tag_2 = LayerTag::new(vec![Tag::new("a"), Tag::new("c")]);
        let test_tag_3 = LayerTag::new(vec![Tag::new("a"), Tag::new("b"), Tag::new("c")]);

        assert_eq!(test_tag_dup.exact_match(&test_tag), true);
        assert_eq!(test_tag.exact_match(&test_tag_2), false);

        assert_eq!(test_tag.partial_match(&test_tag_2), false);
        assert_eq!(test_tag.partial_match(&test_tag_3), true);

        assert_eq!(
            test_tag.same_prefix(&test_tag_2).collect::<Vec<_>>(),
            vec![&Tag::new("a")]
        );

        assert_eq!(
            test_tag.same_prefix(&test_tag_3).collect::<Vec<_>>(),
            vec![&Tag::new("a"), &Tag::new("b")]
        );
    }
}
