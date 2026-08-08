//! 内置属性修饰符：加法修饰与带上限的加法修饰。

use crate::attribute::{AttributeLayer, attribute_set::AttributeSet, modifier::AttributeModifier};

/// 加法修饰符：在指定层上累加属性值；若叠加后最终值超限（为负）则按误差削减。
pub struct AddAttrModifier<T: AttributeSet> {
    attr_type: T::AttributeSetEnum,
    attr_layer: AttributeLayer,
    add_attr_value: f32,
}

impl<T: AttributeSet> AttributeModifier for AddAttrModifier<T> {
    type AttributeSetType = T;

    fn receive_attribute_set(&self, attribute_set: &mut Self::AttributeSetType) {
        let attr = attribute_set.get_attr_mut(self.attr_type);
        let new_final_value = attr.compute_final_value(self.attr_layer, self.add_attr_value);
        if new_final_value > 0.0 {
            attr.add_value(self.attr_layer, self.add_attr_value);
        } else {
            let error = attr.comptue_error_value(self.attr_layer, new_final_value);
            attr.add_value(self.attr_layer, self.add_attr_value - error);
        }
    }
}

impl<T: AttributeSet> AddAttrModifier<T> {
    /// 构造加法修饰符：作用于 `attr_type` 属性的 `attr_layer` 层，加 `value`。
    pub fn new(attr_type: T::AttributeSetEnum, attr_layer: AttributeLayer, value: f32) -> Self {
        Self {
            attr_type,
            attr_layer,
            add_attr_value: value,
        }
    }
}

/// 带上限的加法修饰符：累加后不超过 `max_attr_type` 属性的最终值。
pub struct AddAttrRangeModifier<T: AttributeSet> {
    attr_type: T::AttributeSetEnum,
    max_attr_type: T::AttributeSetEnum,
    attr_layer: AttributeLayer,
    add_attr_value: f32,
}

impl<T: AttributeSet> AttributeModifier for AddAttrRangeModifier<T> {
    type AttributeSetType = T;

    fn receive_attribute_set(&self, attribute_set: &mut Self::AttributeSetType) {
        let attr = attribute_set.get_attr(self.attr_type);
        let max_attr = attribute_set.get_attr(self.max_attr_type);
        let new_final_value = attr.compute_final_value(self.attr_layer, self.add_attr_value);
        let max_final_value = max_attr.get_final_value();
        if new_final_value > max_final_value {
            let error =
                attr.comptue_error_value(self.attr_layer, new_final_value - max_final_value);
            let attr = attribute_set.get_attr_mut(self.attr_type);
            attr.add_value(self.attr_layer, self.add_attr_value - error);
        } else if new_final_value >= 0.0 {
            let attr = attribute_set.get_attr_mut(self.attr_type);
            attr.add_value(self.attr_layer, self.add_attr_value);
        } else {
            let error = attr.comptue_error_value(self.attr_layer, new_final_value);
            let attr = attribute_set.get_attr_mut(self.attr_type);
            attr.add_value(self.attr_layer, self.add_attr_value - error);
        }
    }
}

// TODO: AddAttrModifier and post handle modifier.
// or AddMaxAttrModifier.
