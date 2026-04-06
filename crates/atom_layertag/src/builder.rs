use crate::{count_layertag::CountLayerTag, layertag::LayerTag, tag::Tag};

#[derive(Debug, Default)]
pub struct LayerTagBuilder {
    tags: Vec<Tag>,
}

impl LayerTagBuilder {
    pub fn new() -> Self {
        Self { tags: Vec::new() }
    }

    pub fn add_tag(mut self, tag: Tag) -> Self {
        self.tags.push(tag);
        self
    }

    pub fn build_single(self) -> LayerTag {
        LayerTag::new(self.tags)
    }

    pub fn build_counter(self) -> CountLayerTag {
        let layertag = LayerTag::new(self.tags);
        CountLayerTag::new(layertag)
    }
}
