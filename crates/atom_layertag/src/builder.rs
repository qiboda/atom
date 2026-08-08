use crate::{count_layertag::CountLayerTag, layertag::LayerTag, tag::Tag};

/// [`LayerTag`] / [`CountLayerTag`] 的构建器，支持链式追加标签片段。
#[derive(Debug, Default)]
pub struct LayerTagBuilder {
    tags: Vec<Tag>,
}

impl LayerTagBuilder {
    /// 创建空的构建器。
    pub fn new() -> Self {
        Self { tags: Vec::new() }
    }

    /// 追加一个标签片段，返回自身以支持链式调用。
    pub fn add_tag(mut self, tag: Tag) -> Self {
        self.tags.push(tag);
        self
    }

    /// 构建单个 [`LayerTag`]。
    pub fn build_single(self) -> LayerTag {
        LayerTag::new(self.tags)
    }

    /// 构建带计数的 [`CountLayerTag`]（初始计数为 0）。
    pub fn build_counter(self) -> CountLayerTag {
        let layertag = LayerTag::new(self.tags);
        CountLayerTag::new(layertag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_single() {
        let tag = LayerTagBuilder::new()
            .add_tag(Tag::new("a"))
            .add_tag(Tag::new("b"))
            .build_single();
        assert_eq!(tag.tags().len(), 2);
        assert_eq!(tag.raw_layertag(), "a.b");
    }

    #[test]
    fn test_build_counter() {
        let ct = LayerTagBuilder::new()
            .add_tag(Tag::new("x"))
            .build_counter();
        assert_eq!(ct.count(), 0);
        assert_eq!(ct.layertag().tags().len(), 1);
    }
}
