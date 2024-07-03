use bevy::{reflect::Reflect, utils::HashSet};

use crate::layertag::LayerTag;

pub trait FromTagRegistry {
    fn from_tag_registry() -> Self;
}

/// register layer tag.
/// 1. 至少保证了不会获得无效的LayerTag.
#[derive(Default, Debug, Reflect)]
pub struct LayerTagRegistry {
    layertags: HashSet<LayerTag>,
}

impl LayerTagRegistry {
    pub fn register_raw(&mut self, raw_layertag: &str) {
        let layertag = LayerTag::new_from_raw(raw_layertag);
        self.layertags.insert(layertag);
    }

    pub fn register(&mut self, layertag: LayerTag) {
        self.layertags.insert(layertag);
    }

    pub fn request_from_raw(&self, raw_layertag: &str) -> Option<LayerTag> {
        self.layertags
            .iter()
            .find(|layertag| layertag.raw_layertag() == raw_layertag)
            .cloned()
    }

    pub fn clear(&mut self) {
        self.layertags.clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::{layertag::LayerTag, tag::Tag};

    use super::LayerTagRegistry;

    #[test]
    fn register_layertag() {
        let mut registry = LayerTagRegistry::default();
        let layertag = LayerTag::new(vec![Tag::new("test")]);
        registry.register(layertag);
        assert!(registry.request_from_raw("test").is_some());
        assert!(registry.request_from_raw("alsdkfj").is_none());
    }

    #[test]
    fn request_layertag() {
        let mut registry = LayerTagRegistry::default();
        let layertag = LayerTag::new(vec![Tag::new("test")]);
        registry.register(layertag.clone());
        let new_tag_inst = registry.request_from_raw("test");
        assert_eq!(new_tag_inst, Some(layertag));
        assert!(registry.request_from_raw("safj").is_none());
    }
}
