use ability::attribute::{
    attribute_set::AttributeSet, modifier::AttributeModifier, AttributeLayer,
};

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
    pub fn new(attr_type: T::AttributeSetEnum, attr_layer: AttributeLayer, value: f32) -> Self {
        Self {
            attr_type,
            attr_layer,
            add_attr_value: value,
        }
    }
}

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
