pub mod bundle;
pub mod implement;
pub mod pin;
pub mod plugin;

use bevy::prelude::*;
use pin::EffectNodeExec;
use uuid::Uuid;

use super::{
    context::{EffectGraphContext, InstantEffectNodeMap},
    executor::EffectGraphExecutor,
};

pub trait EffectNode {}

/// all non-system effect node
pub trait InstantEffectNode: Sync + Send {
    fn get_uuid(&self) -> Uuid;

    /// 设置后续执行的输出pin。
    fn push_execute_chain(
        &self,
        context: &EffectGraphContext,
        executor: &mut EffectGraphExecutor,
        input_exec_pin: EffectNodeExec,
        instant_nodes: &Res<InstantEffectNodeMap>,
    );

    fn collect(&self, context: &mut EffectGraphContext);
    fn execute(&self, context: &mut EffectGraphContext);
}

/// all system effect node
pub trait StateEffectNode {}

#[derive(Debug, Component, Default, Copy, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub enum EffectNodeExecuteState {
    #[default]
    Idle,
    Active,
}

/// TODO: change uuid to asset id and entity to entity add asset id
#[derive(Debug, Component, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub enum EffectNodeId {
    Uuid(Uuid),
    Entity(Entity),
}

impl EffectNodeId {
    pub fn from_uuid(uuid: Option<Uuid>) -> Self {
        match uuid {
            Some(uuid) => Self::Uuid(uuid),
            None => Self::Uuid(Uuid::nil()),
        }
    }

    pub fn from_entity(entity: Option<Entity>) -> Self {
        match entity {
            Some(entity) => Self::Entity(entity),
            None => Self::Entity(Entity::PLACEHOLDER),
        }
    }
}

impl From<Entity> for EffectNodeId {
    fn from(entity: Entity) -> Self {
        Self::Entity(entity)
    }
}

impl TryFrom<EffectNodeId> for Entity {
    type Error = &'static str;

    fn try_from(value: EffectNodeId) -> Result<Self, Self::Error> {
        match value {
            EffectNodeId::Entity(entity) => Ok(entity),
            _ => Err("EffectNodeId is not Entity"),
        }
    }
}

impl From<Uuid> for EffectNodeId {
    fn from(uuid: Uuid) -> Self {
        Self::Uuid(uuid)
    }
}

impl TryFrom<EffectNodeId> for Uuid {
    type Error = &'static str;

    fn try_from(value: EffectNodeId) -> Result<Self, Self::Error> {
        match value {
            EffectNodeId::Uuid(uuid) => Ok(uuid),
            _ => Err("EffectNodeId is not Uuid"),
        }
    }
}

impl Default for EffectNodeId {
    fn default() -> Self {
        Self::Uuid(Uuid::nil())
    }
}

// TODO: instead of EffectNodeId uuid
// pub struct EffectNodeAssetId;
// impl EffectNodeAssetId {
//     fn allocate_new_id() -> u32 {
//         static NEXT_ID: AtomicU32 = AtomicU32::new(1);
//         // we increment the value by 1 and fetch the old value
//         // see also: https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html
//         NEXT_ID.fetch_add(1, Ordering::Relaxed)
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_effect_node_id_from_entity() {
        let entity = Entity::from_bits(42);
        let id = EffectNodeId::from(entity);
        assert_eq!(id, EffectNodeId::Entity(entity));
    }

    #[test]
    fn test_effect_node_id_try_into_entity() {
        let entity = Entity::from_bits(42);
        let id = EffectNodeId::Entity(entity);
        let result: Result<Entity, _> = id.try_into();
        assert_eq!(result, Ok(entity));
    }

    #[test]
    fn test_effect_node_id_try_into_entity_fails_for_uuid() {
        let id = EffectNodeId::Uuid(Uuid::nil());
        let result: Result<Entity, _> = id.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_effect_node_id_from_uuid() {
        let uuid = Uuid::new_v4();
        let id = EffectNodeId::from(uuid);
        let result: Result<Uuid, _> = id.try_into();
        assert_eq!(result, Ok(uuid));
    }

    #[test]
    fn test_effect_node_id_from_none_uuid() {
        let id = EffectNodeId::from_uuid(None);
        if let EffectNodeId::Uuid(u) = id {
            assert!(u.is_nil());
        } else {
            panic!("expected Uuid variant");
        }
    }

    #[test]
    fn test_effect_node_id_from_none_entity() {
        let id = EffectNodeId::from_entity(None);
        assert_eq!(id, EffectNodeId::Entity(Entity::PLACEHOLDER));
    }

    #[test]
    fn test_effect_node_id_default() {
        let id = EffectNodeId::default();
        if let EffectNodeId::Uuid(u) = id {
            assert!(u.is_nil());
        } else {
            panic!("expected Uuid variant");
        }
    }
}
