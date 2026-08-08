use std::ops::Deref;

use bevy::prelude::*;

use crate::layertag::LayerTag;

/// 带引用计数的图层标签：在 [`LayerTag`] 基础上附加 `i32` 计数器。
#[derive(Debug, Clone, Reflect)]
pub struct CountLayerTag {
    layertag: LayerTag,
    counter: i32,
}

impl Deref for CountLayerTag {
    type Target = LayerTag;

    fn deref(&self) -> &Self::Target {
        &self.layertag
    }
}

impl CountLayerTag {
    pub(crate) fn new(layertag: LayerTag) -> Self {
        Self {
            layertag,
            counter: 0,
        }
    }

    /// 返回内部 [`LayerTag`] 的引用。
    pub fn layertag(&self) -> &LayerTag {
        &self.layertag
    }
}

impl CountLayerTag {
    /// 计数加一。
    pub fn increase_count(&mut self) {
        self.counter += 1;
    }

    /// 计数减一；减到负数时输出 trace 日志。
    pub fn decrease_count(&mut self) {
        self.counter -= 1;
        if self.counter < 0 {
            trace!(
                "decrease counter to {}, don't should lesser than 0",
                self.counter
            );
        }
    }

    /// 返回当前计数。
    pub fn count(&self) -> i32 {
        self.counter
    }

    /// 将计数重置为零。
    pub fn reset_count(&mut self) {
        self.counter = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{layertag::LayerTag, tag::Tag};

    #[test]
    fn test_new_count_is_zero() {
        let tag = LayerTag::new(vec![Tag::new("test")]);
        let ct = CountLayerTag::new(tag);
        assert_eq!(ct.count(), 0);
    }

    #[test]
    fn test_increase_count() {
        let tag = LayerTag::new(vec![Tag::new("test")]);
        let mut ct = CountLayerTag::new(tag);
        ct.increase_count();
        assert_eq!(ct.count(), 1);
        ct.increase_count();
        assert_eq!(ct.count(), 2);
    }

    #[test]
    fn test_decrease_count() {
        let tag = LayerTag::new(vec![Tag::new("test")]);
        let mut ct = CountLayerTag::new(tag);
        ct.increase_count();
        ct.increase_count();
        ct.decrease_count();
        assert_eq!(ct.count(), 1);
    }

    #[test]
    fn test_reset_count() {
        let tag = LayerTag::new(vec![Tag::new("test")]);
        let mut ct = CountLayerTag::new(tag);
        ct.increase_count();
        ct.increase_count();
        ct.reset_count();
        assert_eq!(ct.count(), 0);
    }
}
