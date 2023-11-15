use bevy::{prelude::Component, reflect::Reflect};

use super::modifier::AttributeModifier;

pub trait AttributeSet: Reflect + Component {
    type AttributeSetEnum;

    fn get_attr_final_value(&self, attribute_set_enum: Self::AttributeSetEnum) -> Option<f32>;

    fn apply_modify(&mut self, modifier: impl AttributeModifier<AttributeSetType = Self>) {
        modifier.receive_attribute_set(self);
    }
}
