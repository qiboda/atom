//! Buff 层数：可叠加的 buff 层数管理。

use bevy::prelude::*;

/// Buff 层数组件：当前层数与上限。
#[derive(Component, Debug, Default, Reflect, Clone)]
#[reflect(Component)]
pub struct BuffLayer {
    layer: i32,
    max_layer: i32,
}

impl BuffLayer {
    /// 创建初始 1 层、上限 `max_layer` 的层数组件。
    pub fn new(max_layer: i32) -> Self {
        Self {
            layer: 1,
            max_layer,
        }
    }
}

impl BuffLayer {
    /// 增加层数（不超过上限）。
    pub fn add_layer(&mut self, layer: i32) {
        assert!(layer > 0, "layer must be greater than 0");
        self.layer += layer;
        self.layer = self.layer.min(self.max_layer);
    }

    /// 减少层数（不低于 0）。
    pub fn remove_layer(&mut self, layer: i32) {
        assert!(layer > 0, "layer must be greater than 0");
        self.layer -= layer;
        self.layer = self.layer.max(0);
    }
}
