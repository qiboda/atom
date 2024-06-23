// use ability::attribute::{attribute_set::AttributeSet, Attribute};
// use bevy::{prelude::Component, reflect::Reflect};

// #[derive(Debug, Default, Reflect)]
// pub(crate) struct ValueAttribute {
//     value: f32,
// }

// impl Attribute for ValueAttribute {
//     fn get_value(&self) -> f32 {
//         &self.value
//     }

//     fn get_value_mut(&mut self) -> &mut f32 {
//         &mut self.value
//     }
// }

// #[derive(Debug, Default, Reflect)]
// pub(crate) struct MoveSpeed {
//     value: f32,
// }

// impl Attribute for MoveSpeed {
//     fn get_value(&self) -> &f32 {
//         &self.value
//     }

//     fn get_value_mut(&mut self) -> &mut f32 {
//         &mut self.value
//     }
// }

// // attribute set derive, to generate AttributeSetEnum with #[attribute] macro];
// #[derive(Debug, Default, Reflect, Component)]
// pub(crate) struct BaseAttributeSet {
//     hp: ValueAttribute,
//     move_speed: MoveSpeed,
// }

// #[derive(Debug, Reflect)]
// pub(crate) enum BaseAttributeSetType {
//     Hp,
//     MoveSpeed,
// }

// impl AttributeSet for BaseAttributeSet {
//     type AttributeSetEnum = BaseAttributeSetType;
//     fn get_attr_final_value(&self, attribute_set_enum: Self::AttributeSetEnum) -> Option<f32> {
//         match attribute_set_enum {
//             BaseAttributeSetType::Hp => Some(self.hp.value),
//             BaseAttributeSetType::MoveSpeed => Some(self.move_speed.value),
//         }
//     }
// }

// #[cfg(test)]
// mod test {
//     use crate::attribute::{BaseAttributeSet, BaseAttributeSetType};
//     use ability::attribute::attribute_set::AttributeSet;

//     struct HpModifier {
//         value: f32,
//     }

//     impl AttributeModifier for HpModifier {
//         type AttributeSetType = BaseAttributeSet;

//         fn receive_attribute_set(&self, attribute_set: &mut Self::AttributeSetType) {
//             *attribute_set.hp.get_value_mut() += self.value;
//         }
//     }

//     #[test]
//     fn test_attr() {
//         let mut attribute_set = BaseAttributeSet::default();
//         let modifier = HpModifier { value: 100.0 };
//         println!(
//             "{:?}",
//             attribute_set.get_attr_final_value(BaseAttributeSetType::Hp)
//         );
//         attribute_set.apply_modify(modifier);
//         println!(
//             "{:?}",
//             attribute_set.get_attr_final_value(BaseAttributeSetType::Hp)
//         );
//     }
// }
