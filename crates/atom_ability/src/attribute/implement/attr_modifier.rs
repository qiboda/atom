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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attribute::Attribute;
    use crate::attribute::implement::attr_base::{BUFF_VALUE_LAYER, ValueAttribute};
    use bevy::prelude::Component;

    #[derive(Debug, Default, Clone, Component)]
    struct TestAttributeSet {
        hp: Box<ValueAttribute>,
        mp: Box<ValueAttribute>,
    }

    #[derive(Debug, Copy, Clone)]
    enum TestAttrType {
        Hp,
        Mp,
    }

    impl AttributeSet for TestAttributeSet {
        type AttributeSetEnum = TestAttrType;

        fn get_attr_final_value(&self, attribute_set_enum: Self::AttributeSetEnum) -> Option<f32> {
            match attribute_set_enum {
                TestAttrType::Hp => Some((*self.hp).get_final_value()),
                TestAttrType::Mp => Some((*self.mp).get_final_value()),
            }
        }

        fn get_attr(&self, attribute_set_enum: Self::AttributeSetEnum) -> &dyn Attribute {
            match attribute_set_enum {
                TestAttrType::Hp => self.hp.as_ref(),
                TestAttrType::Mp => self.mp.as_ref(),
            }
        }

        fn get_attr_mut(
            &mut self,
            attribute_set_enum: Self::AttributeSetEnum,
        ) -> &mut dyn Attribute {
            match attribute_set_enum {
                TestAttrType::Hp => self.hp.as_mut(),
                TestAttrType::Mp => self.mp.as_mut(),
            }
        }
    }

    fn test_set(hp: f32, mp: f32) -> TestAttributeSet {
        TestAttributeSet {
            hp: Box::new(ValueAttribute::new(hp, 0.0, 0.0)),
            mp: Box::new(ValueAttribute::new(mp, 0.0, 0.0)),
        }
    }

    #[test]
    fn add_attr_modifier_adds_value_on_layer() {
        let mut set = test_set(100.0, 0.0);
        let modifier = AddAttrModifier::new(TestAttrType::Hp, BUFF_VALUE_LAYER, 10.0);

        modifier.receive_attribute_set(&mut set);

        assert_eq!(
            set.get_attr_final_value(TestAttrType::Hp),
            Some(110.0),
            "BUFF 层 +10 后最终值应为 110"
        );
    }

    #[test]
    fn add_attr_modifier_negative_dips_below_zero_is_truncated() {
        let mut set = test_set(10.0, 0.0);
        let modifier = AddAttrModifier::new(TestAttrType::Hp, BUFF_VALUE_LAYER, -20.0);

        modifier.receive_attribute_set(&mut set);

        assert_eq!(
            set.get_attr_final_value(TestAttrType::Hp),
            Some(0.0),
            "最终值不允许为负，误差应被削减"
        );
    }

    #[test]
    fn add_attr_modifier_accumulates_on_repeated_application() {
        let mut set = test_set(100.0, 0.0);
        let modifier = AddAttrModifier::new(TestAttrType::Hp, BUFF_VALUE_LAYER, 5.0);

        modifier.receive_attribute_set(&mut set);
        modifier.receive_attribute_set(&mut set);

        assert_eq!(set.get_attr_final_value(TestAttrType::Hp), Some(110.0));
    }

    #[test]
    fn add_attr_modifier_does_not_touch_other_attrs() {
        let mut set = test_set(100.0, 50.0);
        let modifier = AddAttrModifier::new(TestAttrType::Hp, BUFF_VALUE_LAYER, 10.0);

        modifier.receive_attribute_set(&mut set);

        assert_eq!(set.get_attr_final_value(TestAttrType::Mp), Some(50.0));
    }

    #[test]
    fn add_attr_modifier_via_apply_modify() {
        let mut set = test_set(100.0, 0.0);
        let modifier = AddAttrModifier::new(TestAttrType::Hp, BUFF_VALUE_LAYER, 10.0);

        set.apply_modify(modifier);

        assert_eq!(set.get_attr_final_value(TestAttrType::Hp), Some(110.0));
    }

    fn range_modifier(
        attr: TestAttrType,
        max: TestAttrType,
        value: f32,
    ) -> AddAttrRangeModifier<TestAttributeSet> {
        AddAttrRangeModifier {
            attr_type: attr,
            max_attr_type: max,
            attr_layer: BUFF_VALUE_LAYER,
            add_attr_value: value,
        }
    }

    #[test]
    fn add_attr_range_modifier_within_max_adds_full_value() {
        let mut set = test_set(100.0, 200.0);
        let modifier = range_modifier(TestAttrType::Hp, TestAttrType::Mp, 10.0);

        modifier.receive_attribute_set(&mut set);

        assert_eq!(set.get_attr_final_value(TestAttrType::Hp), Some(110.0));
    }

    #[test]
    fn add_attr_range_modifier_over_max_clamps_to_max() {
        let mut set = test_set(100.0, 150.0);
        let modifier = range_modifier(TestAttrType::Hp, TestAttrType::Mp, 100.0);

        modifier.receive_attribute_set(&mut set);

        assert_eq!(
            set.get_attr_final_value(TestAttrType::Hp),
            Some(150.0),
            "累加后超过上限属性最终值时必须削减到上限"
        );
    }

    #[test]
    fn add_attr_range_modifier_negative_dips_below_zero_is_truncated() {
        let mut set = test_set(10.0, 100.0);
        let modifier = range_modifier(TestAttrType::Hp, TestAttrType::Mp, -20.0);

        modifier.receive_attribute_set(&mut set);

        assert_eq!(set.get_attr_final_value(TestAttrType::Hp), Some(0.0));
    }
}
