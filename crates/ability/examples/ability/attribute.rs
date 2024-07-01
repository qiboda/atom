use ability::attribute::{
    attribute_set::AttributeSet, implement::attr_base::ValueAttribute, Attribute,
};
use bevy::{prelude::Component, reflect::Reflect};

// attribute set derive, to generate AttributeSetEnum with #[attribute] macro];
#[derive(Debug, Default, Component)]
pub struct BaseAttributeSet {
    pub hp: Box<ValueAttribute>,
    pub move_speed: Box<ValueAttribute>,
}

#[derive(Debug, Reflect, Copy, Clone, Hash, Eq, PartialEq)]
pub enum BaseAttributeSetType {
    Hp,
    MoveSpeed,
}

impl AttributeSet for BaseAttributeSet {
    type AttributeSetEnum = BaseAttributeSetType;
    fn get_attr_final_value(&self, attribute_set_enum: Self::AttributeSetEnum) -> Option<f32> {
        match attribute_set_enum {
            BaseAttributeSetType::Hp => Some((*self.hp).get_final_value()),
            BaseAttributeSetType::MoveSpeed => Some((*self.move_speed).get_final_value()),
        }
    }

    fn get_attr(
        &self,
        attribute_set_enum: Self::AttributeSetEnum,
    ) -> &dyn ability::attribute::Attribute {
        match attribute_set_enum {
            BaseAttributeSetType::Hp => self.hp.as_ref(),
            BaseAttributeSetType::MoveSpeed => self.move_speed.as_ref(),
        }
    }

    fn get_attr_mut(
        &mut self,
        attribute_set_enum: Self::AttributeSetEnum,
    ) -> &mut dyn ability::attribute::Attribute {
        match attribute_set_enum {
            BaseAttributeSetType::Hp => self.hp.as_mut(),
            BaseAttributeSetType::MoveSpeed => self.move_speed.as_mut(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::attribute::{BaseAttributeSet, BaseAttributeSetType};
    use ability::attribute::attribute_set::AttributeSet;

    struct HpModifier {
        value: f32,
    }

    impl AttributeModifier for HpModifier {
        type AttributeSetType = BaseAttributeSet;

        fn receive_attribute_set(&self, attribute_set: &mut Self::AttributeSetType) {
            *attribute_set.hp.get_value_mut() += self.value;
        }
    }

    #[test]
    fn test_attr() {
        let mut attribute_set = BaseAttributeSet::default();
        let modifier = HpModifier { value: 100.0 };
        println!(
            "{:?}",
            attribute_set.get_attr_final_value(BaseAttributeSetType::Hp)
        );
        attribute_set.apply_modify(modifier);
        println!(
            "{:?}",
            attribute_set.get_attr_final_value(BaseAttributeSetType::Hp)
        );
    }
}
