pub mod hatred;

use bevy::math::Vec3;
use bevy::prelude::*;
use bevy::utils::HashMap;

use crate::unit::{Monster, Npc, Player};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum SensingSystemSet {
    Vision,
    Hearing,
}

#[derive(Default)]
pub struct SensingPlugin;

impl Plugin for SensingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, vision_system.in_set(SensingSystemSet::Vision));
        app.add_systems(PreUpdate, hearing_system.in_set(SensingSystemSet::Hearing));
    }
}

#[derive(Debug, PartialEq)]
pub struct SensingInfo {
    pub distane: f32,
}

impl Default for SensingInfo {
    fn default() -> Self {
        Self { distane: 0.0 }
    }
}

/// 角色视野
///
/// 暂时不要高度限制。
#[derive(Debug, Default, Component)]
pub struct Vision {
    pub range: f32,
    // unit: degree
    pub angle: f32,
    pub direction: Vec3,
    pub sensing_entities: HashMap<Entity, SensingInfo>,
}

impl Vision {
    pub fn new(range: f32, angle: f32, direction: Vec3) -> Self {
        Self {
            range,
            angle,
            direction,
            sensing_entities: HashMap::new(),
        }
    }
}

impl Vision {
    pub fn in_range(&self, distance: f32) -> bool {
        distance <= self.range
    }

    pub fn in_angle(&self, direction: Vec3) -> bool {
        let angle = self.direction.angle_between(direction).to_degrees();
        angle <= self.angle
    }

    pub fn in_vision(&self, distance: f32, direction: Vec3) -> bool {
        self.in_range(distance) && self.in_angle(direction)
    }

    pub fn insert_sensing_entity(&mut self, entity: Entity, info: SensingInfo) {
        self.sensing_entities.insert(entity, info);
    }

    pub fn get_sensing_info(&self, entity: Entity) -> Option<&SensingInfo> {
        self.sensing_entities.get(&entity)
    }
}

fn vision_system(
    mut query: Query<(&GlobalTransform, &mut Vision)>,
    target_query: Query<(Entity, &GlobalTransform), Or<(With<Player>, With<Monster>, With<Npc>)>>,
) {
    for (transform, mut vision) in query.iter_mut() {
        vision.sensing_entities.clear();
        for (target_entity, target_transform) in target_query.iter() {
            let offset = transform.translation() - target_transform.translation();
            let distance = offset.length();
            let direction = offset.normalize();
            if vision.in_vision(distance, direction) {
                vision.insert_sensing_entity(target_entity, SensingInfo { distane: distance });
            }
        }
    }
}

/// 听觉感知
#[derive(Debug, Default, Component)]
pub struct Hearing {
    pub range: f32,
    pub sensing_entities: HashMap<Entity, SensingInfo>,
}

impl Hearing {
    pub fn new(range: f32) -> Self {
        Self {
            range,
            sensing_entities: HashMap::new(),
        }
    }

    pub fn can_hearing(&self, distance: f32) -> bool {
        distance <= self.range
    }

    pub fn insert_sensing_entity(&mut self, entity: Entity, info: SensingInfo) {
        self.sensing_entities.insert(entity, info);
    }

    pub fn get_sensing_info(&self, entity: Entity) -> Option<&SensingInfo> {
        self.sensing_entities.get(&entity)
    }
}

fn hearing_system(
    mut query: Query<(&GlobalTransform, &mut Hearing)>,
    target_query: Query<(Entity, &GlobalTransform), Or<(With<Player>, With<Monster>, With<Npc>)>>,
) {
    for (transform, mut hearing) in query.iter_mut() {
        hearing.sensing_entities.clear();
        for (target_entity, target_transform) in target_query.iter() {
            let offset = transform.translation() - target_transform.translation();
            let distance = offset.length();
            if hearing.can_hearing(distance) {
                hearing.insert_sensing_entity(target_entity, SensingInfo { distane: distance });
            }
        }
    }
}
