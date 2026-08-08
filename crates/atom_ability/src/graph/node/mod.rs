//! Effect Graph 节点：状态节点（占用实体）与即时节点（纯数据）的定义。

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

/// 标记 trait：Effect Graph 节点类型。
pub trait EffectNode {}

/// 即时节点 trait：不占用实体的纯数据节点，通过 UUID 注册。
pub trait InstantEffectNode: Sync + Send {
    /// 返回节点的 UUID。
    fn get_uuid(&self) -> Uuid;

    /// 设置后续执行的输出 pin。
    fn push_execute_chain(
        &self,
        context: &EffectGraphContext,
        executor: &mut EffectGraphExecutor,
        input_exec_pin: EffectNodeExec,
        instant_nodes: &Res<InstantEffectNodeMap>,
    );

    /// 收集节点输出值到上下文。
    fn collect(&self, context: &mut EffectGraphContext);
    /// 执行节点逻辑。
    fn execute(&self, context: &mut EffectGraphContext);
}

/// 状态节点 trait：占用实体、持有运行时状态的系统节点。
pub trait StateEffectNode {}

/// 节点执行状态：用于图整体状态的判定（全部节点 Idle 时图归为非激活）。
#[derive(Debug, Component, Default, Copy, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub enum EffectNodeExecuteState {
    /// 空闲（默认）。
    #[default]
    Idle,
    /// 执行中。
    Active,
}

/// TODO: change uuid to asset id and entity to entity add asset id
/// 节点 ID：即时节点用 [`Uuid`]，状态节点用 [`Entity`]。
#[derive(Debug, Component, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub enum EffectNodeId {
    /// 即时节点（不占实体）。
    Uuid(Uuid),
    /// 状态节点（占实体）。
    Entity(Entity),
}

impl EffectNodeId {
    /// 从可选的 UUID 构造；`None` 时使用 nil UUID。
    pub fn from_uuid(uuid: Option<Uuid>) -> Self {
        match uuid {
            Some(uuid) => Self::Uuid(uuid),
            None => Self::Uuid(Uuid::nil()),
        }
    }

    /// 从可选的实体构造；`None` 时使用 [`Entity::PLACEHOLDER`]。
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
