use bevy::prelude::*;
use bevy::utils::HashMap;
use bitflags::bitflags;

use super::Hearing;
use super::Vision;

use crate::damage::DamageEvent;

bitflags! {
    pub struct HatredFlags: u32 {
        const None = 0b00000000;
        const Damage = 0b00000001;
        const Sensing = 0b00000010;
    }
}

#[derive(Debug, Default)]
pub struct HatredValue {
    pub damage: f32,
    pub sensing: f32,
}

impl HatredValue {
    pub fn get_final_value(&self) -> f32 {
        self.damage + self.sensing
    }
}

/// 仇恨
#[derive(Debug, Default, Component)]
pub struct Hatred {
    pub hates: HashMap<Entity, HatredValue>,
    // 强制仇恨值为设置值。
    pub fixed_hates: HashMap<Entity, f32>,
}

impl Hatred {
    pub fn insert(&mut self, entity: Entity, hate: HatredValue) {
        self.hates.insert(entity, hate);
    }

    pub fn remove(&mut self, entity: Entity) {
        self.hates.remove(&entity);
    }

    pub fn get(&self, entity: Entity) -> Option<&HatredValue> {
        self.hates.get(&entity)
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut HatredValue> {
        self.hates.get_mut(&entity)
    }

    pub fn get_max_hatred_value_entity(&self) -> Option<Entity> {
        self.hates
            .iter()
            .max_by(|(_, lhs), (_, rhs)| {
                lhs.get_final_value()
                    .partial_cmp(&rhs.get_final_value())
                    .unwrap()
            })
            .map(|(entity, _)| *entity)
    }
}

pub const DAMAGE_HATRED_VALUE: f32 = 50.0;

pub fn damage_event_hatred_system(
    mut event_reader: EventReader<DamageEvent>,
    mut query: Query<&mut Hatred>,
) {
    for event in event_reader.read() {
        if let Ok(mut hatred) = query.get_mut(event.target) {
            if let Some(hate) = hatred.get_mut(event.source) {
                hate.damage += DAMAGE_HATRED_VALUE;
            } else {
                hatred.insert(
                    event.source,
                    HatredValue {
                        damage: DAMAGE_HATRED_VALUE,
                        sensing: 0.0,
                    },
                );
            }
        }
    }
}

pub const SENSING_HATRED_MAX: f32 = 30.0;

pub fn sensing_hatred_system(mut query: Query<(&mut Hatred, Option<&Vision>, Option<&Hearing>)>) {
    for (mut hatred, vision, hearing) in query.iter_mut() {
        for (entity, hate) in hatred.hates.iter_mut() {
            hate.sensing = 0.0;
            if let Some(vision) = vision {
                if let Some(info) = vision.get_sensing_info(*entity) {
                    hate.sensing += (info.distane / vision.range) * SENSING_HATRED_MAX;
                }
            }
            if let Some(hearing) = hearing {
                if let Some(info) = hearing.get_sensing_info(*entity) {
                    hate.sensing += (info.distane / hearing.range) * SENSING_HATRED_MAX;
                }
            }
        }
    }
}

pub fn hatred_decrease_system(mut query: Query<&mut Hatred>) {
    for mut hatred in query.iter_mut() {
        for (_, hate) in hatred.hates.iter_mut() {
            hate.damage = (hate.damage - 1.0).max(0.0);
        }
    }
}
