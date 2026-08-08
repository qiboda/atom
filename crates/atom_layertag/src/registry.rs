use bevy::{platform::collections::HashSet, reflect::Reflect};

use crate::layertag::LayerTag;

/// 从图层标签注册表构造自身类型的 trait。
pub trait FromTagRegistry {
    /// 从已注册的 [`LayerTag`] 集合构造自身。
    fn from_tag_registry() -> Self;
}

/// 图层标签注册表：登记合法图层标签，保证请求到的 [`LayerTag`] 均为已注册项。
///
/// 只能通过 [`LayerTagRegistry::register`] / [`LayerTagRegistry::register_raw`]
/// 注册后再用 [`LayerTagRegistry::request_from_raw`] 请求，保证不会获得无效的 [`LayerTag`]。
#[derive(Default, Debug, Reflect)]
pub struct LayerTagRegistry {
    layertags: HashSet<LayerTag>,
}

impl LayerTagRegistry {
    /// 以原始字符串直接注册一个图层标签。
    pub fn register_raw(&mut self, raw_layertag: &str) {
        let layertag = LayerTag::new_from_raw(raw_layertag);
        self.layertags.insert(layertag);
    }

    /// 注册一个 [`LayerTag`]。
    pub fn register(&mut self, layertag: LayerTag) {
        self.layertags.insert(layertag);
    }

    /// 按原始字符串查找已注册的 [`LayerTag`]；未注册时返回 `None`。
    pub fn request_from_raw(&self, raw_layertag: &str) -> Option<LayerTag> {
        self.layertags
            .iter()
            .find(|layertag| layertag.raw_layertag() == raw_layertag)
            .cloned()
    }

    /// 清空注册表。
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
