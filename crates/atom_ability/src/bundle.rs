//! 组件包 trait：统一 spawn 接口与反射标记。

use bevy::{ecs::system::EntityCommands, prelude::Commands, reflect::reflect_trait};

/// 组件包 trait：提供从 `Commands` spawn 自身的能力。
#[reflect_trait]
pub trait BundleTrait {
    /// 通过 `commands` spawn 该组件包。
    fn spawn_bundle<'a>(self, commands: &'a mut Commands) -> EntityCommands<'a>;
}

/// 技能组件包 trait 标记（可反射）。
#[reflect_trait]
pub trait AbilityBundleTrait: BundleTrait {}

/// Buff 组件包 trait 标记（可反射）。
#[reflect_trait]
pub trait BuffBundleTrait: BundleTrait {}
