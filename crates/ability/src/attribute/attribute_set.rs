use bevy::prelude::Component;

use super::{modifier::AttributeModifier, Attribute};

pub trait AttributeSet: Component {
    type AttributeSetEnum: Copy;

    fn get_attr_final_value(&self, attribute_set_enum: Self::AttributeSetEnum) -> Option<f32>;

    fn get_attr(&self, attribute_set_enum: Self::AttributeSetEnum) -> &dyn Attribute;

    fn get_attr_mut(&mut self, attribute_set_enum: Self::AttributeSetEnum) -> &mut dyn Attribute;

    fn apply_modify(&mut self, modifier: impl AttributeModifier<AttributeSetType = Self>) {
        modifier.receive_attribute_set(self);
    }
}
