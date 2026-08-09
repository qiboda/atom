//! 基础属性实现：纯数值属性（[`ValueAttribute`]）与数值+百分比属性
//! （[`ValuePercentAttribute`]）。

use crate::attribute::Attribute;
use crate::attribute::AttributeLayer;
use bevy::log::error;
use bevy::reflect::Reflect;

/// 基础数值层。
pub const BASE_VALUE_LAYER: AttributeLayer = AttributeLayer("base_value");
/// 基础百分比层。
pub const BASE_PERCENT_LAYER: AttributeLayer = AttributeLayer("base_percent");
/// 装备数值层。
pub const ITEM_VALUE_LAYER: AttributeLayer = AttributeLayer("item_value");
/// 装备百分比层。
pub const ITEM_PERCENT_LAYER: AttributeLayer = AttributeLayer("item_percnet");
/// 增益数值层。
pub const BUFF_VALUE_LAYER: AttributeLayer = AttributeLayer("buff_value");
/// 增益百分比层。
pub const BUFF_PERCENT_LAYER: AttributeLayer = AttributeLayer("buff_percent");
/// 无层级（占位，用于整体计算）。
pub const NONE_LAYER: AttributeLayer = AttributeLayer("none");

/// 纯数值属性：最终值 = 各数值层之和。
#[derive(Debug, Default, Reflect, Clone)]
pub struct ValueAttribute {
    base_value: f32,
    item_value: f32,
    buff_value: f32,
    cached_final_value: f32,
}

impl ValueAttribute {
    /// 以初始层值创建属性并计算缓存最终值。
    pub fn new(base_value: f32, item_value: f32, buff_value: f32) -> Self {
        let mut s = Self {
            base_value,
            item_value,
            buff_value,
            cached_final_value: 0.0,
        };

        let final_value = s.compute_final_value(NONE_LAYER, 0.0);
        s.set_final_value(final_value);
        s
    }
}

impl Attribute for ValueAttribute {
    fn get_value(&self, layer: AttributeLayer) -> Option<f32> {
        match layer {
            BASE_VALUE_LAYER => Some(self.base_value),
            ITEM_VALUE_LAYER => Some(self.item_value),
            BUFF_VALUE_LAYER => Some(self.buff_value),
            _ => None,
        }
    }

    fn set_value(&mut self, layer: AttributeLayer, value: f32) {
        match layer {
            BASE_VALUE_LAYER => self.base_value = value,
            ITEM_VALUE_LAYER => self.item_value = value,
            BUFF_VALUE_LAYER => self.buff_value = value,
            _ => error!("set_value error: layer not found!"),
        }

        let final_value = self.compute_final_value(NONE_LAYER, 0.0);
        self.set_final_value(final_value);
    }

    fn add_value(&mut self, layer: AttributeLayer, value: f32) {
        match layer {
            BASE_VALUE_LAYER => self.base_value += value,
            ITEM_VALUE_LAYER => self.item_value += value,
            BUFF_VALUE_LAYER => self.buff_value += value,
            _ => error!("set_value error: layer not found!"),
        }

        let final_value = self.compute_final_value(NONE_LAYER, 0.0);
        self.set_final_value(final_value);
    }

    fn get_final_value(&self) -> f32 {
        self.cached_final_value
    }

    fn compute_final_value(&self, layer: AttributeLayer, layer_value: f32) -> f32 {
        match layer {
            BASE_VALUE_LAYER => self.base_value + self.item_value + self.buff_value + layer_value,
            ITEM_VALUE_LAYER => self.base_value + self.item_value + self.buff_value + layer_value,
            BUFF_VALUE_LAYER => self.base_value + self.item_value + self.buff_value + layer_value,
            _ => self.base_value + self.item_value + self.buff_value,
        }
    }

    fn set_final_value(&mut self, final_value: f32) {
        self.cached_final_value = final_value;
    }

    fn comptue_error_value(&self, _layer: AttributeLayer, final_value_error: f32) -> f32 {
        final_value_error
    }
}

/// 纯数值属性的别名（与数值百分比属性区分的历史命名）。
pub type PercentAttribute = ValueAttribute;

/// 数值 + 百分比属性：最终值 = Σ(层值 × (1 + 层百分比))。
#[derive(Debug, Default, Reflect)]
pub struct ValuePercentAttribute {
    base_value: f32,
    item_value: f32,
    buff_value: f32,
    base_percent: f32,
    item_percent: f32,
    buff_percent: f32,
    cached_final_value: f32,
}

impl Attribute for ValuePercentAttribute {
    fn get_value(&self, layer: AttributeLayer) -> Option<f32> {
        match layer {
            BASE_VALUE_LAYER => Some(self.base_value),
            ITEM_VALUE_LAYER => Some(self.item_value),
            BUFF_VALUE_LAYER => Some(self.buff_value),
            BASE_PERCENT_LAYER => Some(self.base_percent),
            ITEM_PERCENT_LAYER => Some(self.item_percent),
            BUFF_PERCENT_LAYER => Some(self.buff_percent),
            _ => None,
        }
    }

    fn set_value(&mut self, layer: AttributeLayer, value: f32) {
        match layer {
            BASE_VALUE_LAYER => self.base_value = value,
            ITEM_VALUE_LAYER => self.item_value = value,
            BUFF_VALUE_LAYER => self.buff_value = value,
            BASE_PERCENT_LAYER => self.base_percent = value,
            ITEM_PERCENT_LAYER => self.item_percent = value,
            BUFF_PERCENT_LAYER => self.buff_percent = value,
            _ => {
                error!("set_value error: layer not found!")
            }
        }

        let final_value = self.compute_final_value(NONE_LAYER, 0.0);
        self.set_final_value(final_value);
    }

    fn add_value(&mut self, layer: AttributeLayer, value: f32) {
        match layer {
            BASE_VALUE_LAYER => self.base_value += value,
            ITEM_VALUE_LAYER => self.item_value += value,
            BUFF_VALUE_LAYER => self.buff_value += value,
            BASE_PERCENT_LAYER => self.base_percent += value,
            ITEM_PERCENT_LAYER => self.item_percent += value,
            BUFF_PERCENT_LAYER => self.buff_percent += value,
            _ => {
                error!("add_value error: layer not found!")
            }
        }

        let final_value = self.compute_final_value(NONE_LAYER, 0.0);
        self.set_final_value(final_value);
    }

    fn get_final_value(&self) -> f32 {
        self.cached_final_value
    }

    fn compute_final_value(&self, layer: AttributeLayer, layer_value: f32) -> f32 {
        match layer {
            BASE_VALUE_LAYER => {
                (self.base_value + layer_value) * (1.0 + self.base_percent)
                    + self.item_value * (1.0 + self.item_percent)
                    + self.buff_value * (1.0 + self.buff_percent)
            }
            ITEM_VALUE_LAYER => {
                self.base_value * (1.0 + self.base_percent)
                    + (self.item_value + layer_value) * (1.0 + self.item_percent)
                    + self.buff_value * (1.0 + self.buff_percent)
            }
            BUFF_VALUE_LAYER => {
                self.base_value * (1.0 + self.base_percent)
                    + self.item_value * (1.0 + self.item_percent)
                    + (self.buff_value + layer_value) * (1.0 + self.buff_percent)
            }
            BASE_PERCENT_LAYER => {
                self.base_value * (1.0 + self.base_percent + layer_value)
                    + self.item_value * (1.0 + self.item_percent)
                    + self.buff_value * (1.0 + self.buff_percent)
            }
            ITEM_PERCENT_LAYER => {
                self.base_value * (1.0 + self.base_percent)
                    + self.item_value * (1.0 + self.item_percent + layer_value)
                    + self.buff_value * (1.0 + self.buff_percent)
            }
            BUFF_PERCENT_LAYER => {
                self.base_value * (1.0 + self.base_percent)
                    + self.item_value * (1.0 + self.item_percent)
                    + self.buff_value * (1.0 + self.buff_percent + layer_value)
            }
            _ => {
                self.base_value * (1.0 + self.base_percent)
                    + self.item_value * (1.0 + self.item_percent)
                    + self.buff_value * (1.0 + self.buff_percent)
            }
        }
    }

    fn set_final_value(&mut self, final_value: f32) {
        self.cached_final_value = final_value;
    }

    fn comptue_error_value(&self, layer: AttributeLayer, final_value_error: f32) -> f32 {
        match layer {
            BASE_VALUE_LAYER => final_value_error / (1.0 + self.base_percent),
            ITEM_VALUE_LAYER => final_value_error / (1.0 + self.item_percent),
            BUFF_VALUE_LAYER => final_value_error / (1.0 + self.buff_percent),
            BASE_PERCENT_LAYER => final_value_error / self.base_value - 1.0,
            ITEM_PERCENT_LAYER => final_value_error / self.item_value - 1.0,
            BUFF_PERCENT_LAYER => final_value_error / self.buff_value - 1.0,
            _ => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_final_value() {
        let attr = ValueAttribute::new(1.0, 2.0, 3.0);
        assert_eq!(attr.get_final_value(), 6.0);
    }

    #[test]
    fn test_get_value() {
        let attr = ValueAttribute::new(1.0, 2.0, 3.0);
        assert_eq!(attr.get_value(BASE_VALUE_LAYER), Some(1.0));
        assert_eq!(attr.get_value(ITEM_VALUE_LAYER), Some(2.0));
        assert_eq!(attr.get_value(BUFF_VALUE_LAYER), Some(3.0));
    }

    #[test]
    fn test_value_attribute_set_value() {
        let mut attr = ValueAttribute::new(10.0, 0.0, 0.0);
        assert_eq!(attr.get_final_value(), 10.0);
        attr.set_value(ITEM_VALUE_LAYER, 5.0);
        assert_eq!(attr.get_final_value(), 15.0);
    }

    #[test]
    fn test_value_attribute_add_value() {
        let mut attr = ValueAttribute::new(10.0, 0.0, 0.0);
        attr.add_value(BUFF_VALUE_LAYER, 3.0);
        assert_eq!(attr.get_final_value(), 13.0);
    }

    #[test]
    fn test_value_percent_attribute_basic() {
        let mut attr = ValuePercentAttribute::default();
        attr.set_value(BASE_VALUE_LAYER, 100.0);
        // final = 100 * (1+0) + 0 * (1+0) + 0 * (1+0) = 100
        assert_eq!(attr.get_final_value(), 100.0);
    }

    #[test]
    fn test_value_percent_attribute_with_percent() {
        let mut attr = ValuePercentAttribute::default();
        attr.set_value(BASE_VALUE_LAYER, 100.0);
        attr.set_value(BASE_PERCENT_LAYER, 0.5);
        // final = 100 * (1+0.5) + 0 + 0 = 150
        assert_eq!(attr.get_final_value(), 150.0);
    }

    #[test]
    fn test_value_percent_attribute_all_layers() {
        let mut attr = ValuePercentAttribute::default();
        attr.set_value(BASE_VALUE_LAYER, 100.0);
        attr.set_value(BASE_PERCENT_LAYER, 0.1);
        attr.set_value(ITEM_VALUE_LAYER, 50.0);
        attr.set_value(ITEM_PERCENT_LAYER, 0.2);
        attr.set_value(BUFF_VALUE_LAYER, 20.0);
        attr.set_value(BUFF_PERCENT_LAYER, 0.5);
        // final = 100*(1+0.1) + 50*(1+0.2) + 20*(1+0.5) = 110 + 60 + 30 = 200
        assert_eq!(attr.get_final_value(), 200.0);
    }

    #[test]
    fn test_value_percent_compute_error_value() {
        let mut attr = ValuePercentAttribute::default();
        attr.set_value(BASE_VALUE_LAYER, 100.0);
        attr.set_value(BASE_PERCENT_LAYER, 0.5);
        // comptue_error_value for BASE_VALUE_LAYER: error / (1 + 0.5) = error / 1.5
        let err = attr.comptue_error_value(BASE_VALUE_LAYER, 15.0);
        assert!((err - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_get_value_unknown_layer() {
        let attr = ValueAttribute::new(1.0, 2.0, 3.0);
        assert_eq!(attr.get_value(NONE_LAYER), None);
    }

    #[test]
    fn test_value_attribute_set_value_unknown_layer_keeps_final() {
        let mut attr = ValueAttribute::new(1.0, 2.0, 3.0);
        // 未知层级：仅记录 error，不改变任何层值，最终值保持不变。
        attr.set_value(NONE_LAYER, 99.0);
        assert_eq!(attr.get_final_value(), 6.0);
        assert_eq!(attr.get_value(BASE_VALUE_LAYER), Some(1.0));
    }

    #[test]
    fn test_value_attribute_add_value_unknown_layer_keeps_final() {
        let mut attr = ValueAttribute::new(1.0, 2.0, 3.0);
        attr.add_value(NONE_LAYER, 99.0);
        assert_eq!(attr.get_final_value(), 6.0);
        assert_eq!(attr.get_value(ITEM_VALUE_LAYER), Some(2.0));
    }

    #[test]
    fn test_value_attribute_compute_final_value_with_layer_value() {
        let attr = ValueAttribute::new(1.0, 2.0, 3.0);
        // 任一数值层都按 三值之和 + layer_value 计算。
        assert_eq!(attr.compute_final_value(BASE_VALUE_LAYER, 10.0), 16.0);
        assert_eq!(attr.compute_final_value(ITEM_VALUE_LAYER, 10.0), 16.0);
        assert_eq!(attr.compute_final_value(BUFF_VALUE_LAYER, 10.0), 16.0);
        // 未知层级忽略 layer_value。
        assert_eq!(attr.compute_final_value(NONE_LAYER, 10.0), 6.0);
    }

    #[test]
    fn test_value_attribute_set_final_value_and_error_value() {
        let mut attr = ValueAttribute::new(1.0, 2.0, 3.0);
        attr.set_final_value(42.0);
        assert_eq!(attr.get_final_value(), 42.0);

        // 纯数值属性误差直接透传。
        assert_eq!(attr.comptue_error_value(BASE_VALUE_LAYER, 5.0), 5.0);
        assert_eq!(attr.comptue_error_value(NONE_LAYER, 7.0), 7.0);
    }

    #[test]
    fn test_value_percent_get_value_all_layers() {
        let mut attr = ValuePercentAttribute::default();
        attr.set_value(BASE_VALUE_LAYER, 1.0);
        attr.set_value(ITEM_VALUE_LAYER, 2.0);
        attr.set_value(BUFF_VALUE_LAYER, 3.0);
        attr.set_value(BASE_PERCENT_LAYER, 0.1);
        attr.set_value(ITEM_PERCENT_LAYER, 0.2);
        attr.set_value(BUFF_PERCENT_LAYER, 0.3);

        assert_eq!(attr.get_value(BASE_VALUE_LAYER), Some(1.0));
        assert_eq!(attr.get_value(ITEM_VALUE_LAYER), Some(2.0));
        assert_eq!(attr.get_value(BUFF_VALUE_LAYER), Some(3.0));
        assert_eq!(attr.get_value(BASE_PERCENT_LAYER), Some(0.1));
        assert_eq!(attr.get_value(ITEM_PERCENT_LAYER), Some(0.2));
        assert_eq!(attr.get_value(BUFF_PERCENT_LAYER), Some(0.3));
        assert_eq!(attr.get_value(NONE_LAYER), None);
    }

    #[test]
    fn test_value_percent_add_value_all_layers() {
        let mut attr = ValuePercentAttribute::default();
        attr.set_value(BASE_VALUE_LAYER, 100.0);
        attr.add_value(ITEM_VALUE_LAYER, 10.0);
        attr.add_value(ITEM_PERCENT_LAYER, 0.5);
        attr.add_value(BUFF_VALUE_LAYER, 20.0);
        attr.add_value(BUFF_PERCENT_LAYER, 0.25);
        // final = 100*(1+0) + 10*(1+0.5) + 20*(1+0.25) = 100 + 15 + 25 = 140
        assert_eq!(attr.get_final_value(), 140.0);
    }

    #[test]
    fn test_value_percent_set_unknown_layer_keeps_final() {
        let mut attr = ValuePercentAttribute::default();
        attr.set_value(BASE_VALUE_LAYER, 100.0);
        let before = attr.get_final_value();

        attr.set_value(NONE_LAYER, 999.0);
        assert_eq!(attr.get_final_value(), before);

        attr.add_value(NONE_LAYER, 999.0);
        assert_eq!(attr.get_final_value(), before);
    }

    #[test]
    fn test_value_percent_compute_final_value_layer_branches() {
        let mut attr = ValuePercentAttribute::default();
        attr.set_value(BASE_VALUE_LAYER, 100.0);
        attr.set_value(BASE_PERCENT_LAYER, 0.1);
        attr.set_value(ITEM_VALUE_LAYER, 50.0);
        attr.set_value(ITEM_PERCENT_LAYER, 0.2);
        attr.set_value(BUFF_VALUE_LAYER, 20.0);
        attr.set_value(BUFF_PERCENT_LAYER, 0.5);
        // 基线：110 + 60 + 30 = 200
        assert_eq!(attr.compute_final_value(NONE_LAYER, 0.0), 200.0);

        // BASE_VALUE_LAYER: (100+10)*(1.1) + 60 + 30 = 121 + 90 = 211
        assert_eq!(attr.compute_final_value(BASE_VALUE_LAYER, 10.0), 211.0);
        // ITEM_VALUE_LAYER: 110 + (50+10)*1.2 + 30 = 110 + 72 + 30 = 212
        assert_eq!(attr.compute_final_value(ITEM_VALUE_LAYER, 10.0), 212.0);
        // BUFF_VALUE_LAYER: 110 + 60 + (20+10)*1.5 = 110 + 60 + 45 = 215
        assert_eq!(attr.compute_final_value(BUFF_VALUE_LAYER, 10.0), 215.0);
        // BASE_PERCENT_LAYER: 100*(1.1+0.5) + 60 + 30 = 160 + 90 = 250
        assert_eq!(attr.compute_final_value(BASE_PERCENT_LAYER, 0.5), 250.0);
        // ITEM_PERCENT_LAYER: 110 + 50*(1.2+0.5) + 30 = 110 + 85 + 30 = 225
        assert_eq!(attr.compute_final_value(ITEM_PERCENT_LAYER, 0.5), 225.0);
        // BUFF_PERCENT_LAYER: 110 + 60 + 20*(1.5+0.5) = 110 + 60 + 40 = 210
        assert_eq!(attr.compute_final_value(BUFF_PERCENT_LAYER, 0.5), 210.0);
    }

    #[test]
    fn test_value_percent_compute_error_value_all_branches() {
        let mut attr = ValuePercentAttribute::default();
        attr.set_value(BASE_VALUE_LAYER, 100.0);
        attr.set_value(BASE_PERCENT_LAYER, 0.5);
        attr.set_value(ITEM_VALUE_LAYER, 50.0);
        attr.set_value(ITEM_PERCENT_LAYER, 0.25);
        attr.set_value(BUFF_VALUE_LAYER, 20.0);
        attr.set_value(BUFF_PERCENT_LAYER, 0.0);

        // 数值层误差除以 (1 + 对应百分比)。
        assert_eq!(attr.comptue_error_value(BASE_VALUE_LAYER, 30.0), 20.0);
        assert_eq!(attr.comptue_error_value(ITEM_VALUE_LAYER, 12.5), 10.0);
        assert_eq!(attr.comptue_error_value(BUFF_VALUE_LAYER, 20.0), 20.0);
        // 百分比层误差反推：error/base - 1（浮点除法需容差）。
        assert!((attr.comptue_error_value(BASE_PERCENT_LAYER, 120.0) - 0.2).abs() < 1e-6);
        assert!((attr.comptue_error_value(ITEM_PERCENT_LAYER, 75.0) - 0.5).abs() < 1e-6);
        assert!((attr.comptue_error_value(BUFF_PERCENT_LAYER, 30.0) - 0.5).abs() < 1e-6);
        // 未知层级返回 0。
        assert_eq!(attr.comptue_error_value(NONE_LAYER, 99.0), 0.0);
    }
}
