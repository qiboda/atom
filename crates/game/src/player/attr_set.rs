use ability::attribute::{attribute_set::AttributeSet, modifier::AttributeModifier, Attribute};
use bevy::{prelude::Component, reflect::Reflect};

use crate::attr::attr_base::{ValueAttribute, ValuePercentAttribute};

#[derive(Debug, Reflect, Copy, Clone, Hash, Eq, PartialEq)]
pub enum CharacterAttributeType {
    Hp,
    MaxHp,
    MoveSpeed,
    MaxMoveSpped,
}

// attribute set derive, to generate AttributeSetEnum with #[attribute] macro];
#[derive(Debug, Default, Component)]
pub struct CharacterAttributeSet {
    hp: Box<ValueAttribute>,
    max_hp: Box<ValueAttribute>,
    move_speed: Box<ValuePercentAttribute>,
    max_move_spped: Box<ValuePercentAttribute>,
}

impl AttributeSet for CharacterAttributeSet {
    type AttributeSetEnum = CharacterAttributeType;
    fn get_attr_final_value(&self, attribute_set_enum: Self::AttributeSetEnum) -> Option<f32> {
        match attribute_set_enum {
            CharacterAttributeType::Hp => Some(self.hp.get_final_value()),
            CharacterAttributeType::MoveSpeed => Some(self.move_speed.get_final_value()),
            CharacterAttributeType::MaxHp => Some(self.max_hp.get_final_value()),
            CharacterAttributeType::MaxMoveSpped => Some(self.max_move_spped.get_final_value()),
        }
    }

    fn apply_modify(&mut self, modifier: impl AttributeModifier<AttributeSetType = Self>) {
        modifier.receive_attribute_set(self);
    }

    fn get_attr_mut(&mut self, attribute_set_enum: Self::AttributeSetEnum) -> &mut dyn Attribute {
        match attribute_set_enum {
            CharacterAttributeType::Hp => self.hp.as_mut(),
            CharacterAttributeType::MoveSpeed => self.move_speed.as_mut(),
            CharacterAttributeType::MaxHp => self.max_hp.as_mut(),
            CharacterAttributeType::MaxMoveSpped => self.max_move_spped.as_mut(),
        }
    }

    fn get_attr(&self, attribute_set_enum: Self::AttributeSetEnum) -> &dyn Attribute {
        match attribute_set_enum {
            CharacterAttributeType::Hp => self.hp.as_ref(),
            CharacterAttributeType::MoveSpeed => self.move_speed.as_ref(),
            CharacterAttributeType::MaxHp => self.max_hp.as_ref(),
            CharacterAttributeType::MaxMoveSpped => self.max_move_spped.as_ref(),
        }
    }
}
