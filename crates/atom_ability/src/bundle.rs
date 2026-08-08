//! 效果组件包 trait：反射调度的效果 spawn 接口。

use bevy::{
    ecs::system::EntityCommands,
    prelude::Commands,
    reflect::reflect_trait,
    scene::{Scene, prelude::CommandsSceneExt},
};

/// 效果组件包 trait：通过反射从 `Box<dyn Reflect>` 还原并 spawn 效果场景。
#[reflect_trait]
pub trait EffectBundleTrait {
    /// 构建效果场景。
    fn build_scene(&self) -> Box<dyn Scene>;
    /// 通过 `commands` 以 BSN 场景方式 spawn 该效果。
    fn spawn_scene<'a>(&self, commands: &'a mut Commands) -> EntityCommands<'a> {
        commands.spawn_scene(self.build_scene())
    }
}
