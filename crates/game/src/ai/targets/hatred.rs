/// 仇恨系统
///
/// 仇恨系统是一个目标系统，它会根据伤害和感知来计算仇恨值。
/// 仇恨系统也包含了友好度系统，友好度系统是仇恨系统的反向。
/// 因此仇恨值越低，友好度越高。0是一个中立值。低于0是偏向友好。
use bevy::prelude::*;
use bevy::utils::HashMap;
use bitflags::bitflags;
use datatables::tables_system_param::TableReader;
use datatables::unit::RelationShipType;
use datatables::unit::TbMonsterRow;
use datatables::unit::TbNpcRow;
use datatables::unit::TbPlayerRow;
use datatables::unit::TbRelationShip;

use crate::damage::DamageEvent;

use super::sensing::Hearing;
use super::sensing::Vision;
use super::TargetsSystemSet;

#[derive(Debug, Default, SystemSet, Hash, PartialEq, Eq, Clone, Copy)]
pub struct HatredSystemSet;

#[derive(Debug, Default)]
pub struct HatredPlugin;

impl Plugin for HatredPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(PreUpdate, HatredSystemSet.in_set(TargetsSystemSet::Hatred))
            .add_systems(PreUpdate, sensing_hatred_system.in_set(HatredSystemSet))
            .add_systems(
                PreUpdate,
                (damage_event_hatred_system, hatred_decrease_system).in_set(HatredSystemSet),
            );
    }
}

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
    // TODO 改为最大堆，这样可以快速找到最大值。是否需要最小堆？
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
            .filter(|(_, hatred)| hatred.get_final_value() > 0.0)
            .max_by(|(_, lhs), (_, rhs)| {
                lhs.get_final_value()
                    .partial_cmp(&rhs.get_final_value())
                    .unwrap()
            })
            .map(|(entity, _)| *entity)
    }

    pub fn get_max_friendly_value_entity(&self) -> Option<Entity> {
        self.hates
            .iter()
            .filter(|(_, hatred)| hatred.get_final_value() < 0.0)
            .min_by(|(_, lhs), (_, rhs)| {
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

#[allow(clippy::type_complexity)]
pub fn sensing_hatred_system(
    mut query: Query<(
        &mut Hatred,
        Option<&TbPlayerRow>,
        Option<&TbNpcRow>,
        Option<&TbMonsterRow>,
        Option<&Vision>,
        Option<&Hearing>,
    )>,
    mut target_query: Query<(
        Entity,
        Option<&TbPlayerRow>,
        Option<&TbNpcRow>,
        Option<&TbMonsterRow>,
    )>,
    relationship_reader: TableReader<TbRelationShip>,
) {
    for (mut hatred, tbplayer_row, tbnpc_row, tbmonster_row, vision, hearing) in query.iter_mut() {
        let camp = if let Some(tbplayer_row) = tbplayer_row {
            tbplayer_row.data.as_ref().unwrap().camp
        } else if let Some(tbnpc_row) = tbnpc_row {
            tbnpc_row.data.as_ref().unwrap().camp
        } else if let Some(tbmonster_row) = tbmonster_row {
            tbmonster_row.data.as_ref().unwrap().camp
        } else {
            -1
        };

        for (target_entity, hate) in hatred.hates.iter_mut() {
            let (target_entity, target_tbplayer_row, target_tbnpc_row, target_tbmonster_row) =
                target_query.get_mut(*target_entity).unwrap();
            let target_camp = if let Some(tbplayer_row) = target_tbplayer_row {
                tbplayer_row.data.as_ref().unwrap().camp
            } else if let Some(tbnpc_row) = target_tbnpc_row {
                tbnpc_row.data.as_ref().unwrap().camp
            } else if let Some(tbmonster_row) = target_tbmonster_row {
                tbmonster_row.data.as_ref().unwrap().camp
            } else {
                -1
            };

            let relationship_row = relationship_reader
                .get_row_by_key(&(camp, target_camp))
                .unwrap();

            hate.sensing = 0.0;

            match relationship_row.relationship_type {
                RelationShipType::None => {}
                RelationShipType::Hostility => {
                    if let Some(vision) = vision {
                        if let Some(info) = vision.get_sensing_info(target_entity) {
                            hate.sensing += (info.distance / vision.range) * SENSING_HATRED_MAX;
                        }
                    }
                    if let Some(hearing) = hearing {
                        if let Some(info) = hearing.get_sensing_info(target_entity) {
                            hate.sensing += (info.distance / hearing.range) * SENSING_HATRED_MAX;
                        }
                    }
                }
                RelationShipType::Friendly => {
                    if let Some(vision) = vision {
                        if let Some(info) = vision.get_sensing_info(target_entity) {
                            hate.sensing -= (info.distance / vision.range) * SENSING_HATRED_MAX;
                        }
                    }
                    if let Some(hearing) = hearing {
                        if let Some(info) = hearing.get_sensing_info(target_entity) {
                            hate.sensing -= (info.distance / hearing.range) * SENSING_HATRED_MAX;
                        }
                    }
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
