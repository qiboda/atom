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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_at_one_layer() {
        let buff_layer = BuffLayer::new(5);
        assert_eq!(buff_layer.layer, 1);
        assert_eq!(buff_layer.max_layer, 5);
    }

    #[test]
    fn add_layer_increases_below_max() {
        let mut buff_layer = BuffLayer::new(5);
        buff_layer.add_layer(2);
        assert_eq!(buff_layer.layer, 3);
    }

    #[test]
    fn add_layer_clamps_at_max() {
        let mut buff_layer = BuffLayer::new(5);
        buff_layer.add_layer(2);
        buff_layer.add_layer(3);
        assert_eq!(buff_layer.layer, 5);
        buff_layer.add_layer(1);
        assert_eq!(buff_layer.layer, 5, "超过上限后不得继续增加");
    }

    #[test]
    fn add_layer_multi_shot_reaches_max() {
        let mut buff_layer = BuffLayer::new(3);
        buff_layer.add_layer(10);
        assert_eq!(buff_layer.layer, 3);
    }

    #[test]
    fn remove_layer_decreases_above_zero() {
        let mut buff_layer = BuffLayer::new(10);
        buff_layer.add_layer(4);
        assert_eq!(buff_layer.layer, 5);

        buff_layer.remove_layer(2);
        assert_eq!(buff_layer.layer, 3);
    }

    #[test]
    fn remove_layer_clamps_at_zero() {
        let mut buff_layer = BuffLayer::new(10);
        buff_layer.remove_layer(5);
        assert_eq!(buff_layer.layer, 0, "减少层数不得低于 0");
        buff_layer.remove_layer(1);
        assert_eq!(buff_layer.layer, 0);
    }

    #[test]
    fn add_and_remove_roundtrip_restores_initial() {
        let mut buff_layer = BuffLayer::new(10);
        buff_layer.add_layer(3);
        buff_layer.remove_layer(3);
        assert_eq!(buff_layer.layer, 1);
    }

    #[test]
    #[should_panic(expected = "layer must be greater than 0")]
    fn add_layer_zero_panics() {
        let mut buff_layer = BuffLayer::new(5);
        buff_layer.add_layer(0);
    }

    #[test]
    #[should_panic(expected = "layer must be greater than 0")]
    fn add_layer_negative_panics() {
        let mut buff_layer = BuffLayer::new(5);
        buff_layer.add_layer(-1);
    }

    #[test]
    #[should_panic(expected = "layer must be greater than 0")]
    fn remove_layer_zero_panics() {
        let mut buff_layer = BuffLayer::new(5);
        buff_layer.remove_layer(0);
    }

    #[test]
    #[should_panic(expected = "layer must be greater than 0")]
    fn remove_layer_negative_panics() {
        let mut buff_layer = BuffLayer::new(5);
        buff_layer.remove_layer(-2);
    }
}
