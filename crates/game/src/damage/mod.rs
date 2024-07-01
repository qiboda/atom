use ability::attribute::{
    attribute_set::AttributeSet,
    implement::{attr_base::BASE_VALUE_LAYER, attr_modifier::AddAttrModifier},
    AttributeLayer,
};
use bevy::prelude::*;

use crate::unit::attr_set::{CharacterAttributeSet, CharacterAttributeType};

#[derive(Event)]
pub struct DamageEvent {
    pub target: Entity,
    pub source: Entity,
    pub damage_layer: AttributeLayer,
    pub damage: f32,
}

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEvent>()
            .add_systems(PostUpdate, apply_damage);
    }
}

fn apply_damage(
    mut damage_events: EventReader<DamageEvent>,
    mut attr_set: Query<&mut CharacterAttributeSet>,
) {
    for event in damage_events.read() {
        if let Ok(mut target_attr) = attr_set.get_mut(event.target) {
            let modifier = AddAttrModifier::<CharacterAttributeSet>::new(
                CharacterAttributeType::Hp,
                BASE_VALUE_LAYER,
                -event.damage,
            );
            target_attr.apply_modify(modifier);
        }
    }
}
