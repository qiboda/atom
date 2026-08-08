//! Effect 场景模板：BSN（Bevy Scene Notation）构造 effect 状态层标签组件。

use bevy::prelude::*;

use super::tag::{
    EffectAbortDisableLayerTagContainer, EffectAbortRequiredLayerTagContainer,
    EffectAddedLayerTagContainer, EffectRemovedLayerTagContainer,
    EffectStartDisableLayerTagContainer, EffectStartRequiredLayerTagContainer,
};

/// 依据默认状态构造 effect 状态层标签组件场景。
///
/// 与迁移前 `EffectStartTagBundle`/`EffectAbortTagBundle` 产物一致：
/// 6 个状态层标签容器（开始/中断各自所需与禁用、开始时的增删集合）。
/// 若需从数据表构造容器内容，可仿照 [`crate::buff::bundle::spawn_buff`]
/// 接受参数并用 `template_value` 注入。
pub fn spawn_effect() -> impl Scene {
    bsn! {
        EffectStartRequiredLayerTagContainer
        EffectStartDisableLayerTagContainer
        EffectAddedLayerTagContainer
        EffectRemovedLayerTagContainer
        EffectAbortRequiredLayerTagContainer
        EffectAbortDisableLayerTagContainer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atom_layertag::container_op::LayerTagContainer;
    use bevy::{MinimalPlugins, asset::AssetPlugin, scene::ScenePlugin};

    #[test]
    fn spawn_effect_produces_six_tag_containers() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default(), ScenePlugin));
        app.add_systems(Update, |mut commands: Commands| {
            commands.spawn_scene(spawn_effect());
        });
        app.update();

        let world = app.world_mut();
        let mut query = world.query::<(
            &EffectStartRequiredLayerTagContainer,
            &EffectStartDisableLayerTagContainer,
            &EffectAddedLayerTagContainer,
            &EffectRemovedLayerTagContainer,
            &EffectAbortRequiredLayerTagContainer,
            &EffectAbortDisableLayerTagContainer,
        )>();
        let item = query
            .iter(world)
            .next()
            .expect("spawn_effect 场景应产生一个 effect 实体");

        let (start_required, start_disable, added, removed, abort_required, abort_disable) = item;

        // 全部容器默认构造为空且存在。
        assert!(start_required.0.iter_layertag().next().is_none());
        assert!(start_disable.0.iter_layertag().next().is_none());
        assert!(added.layer_tag_container.iter_layertag().next().is_none());
        assert!(removed.layer_tag_container.iter_layertag().next().is_none());
        assert!(abort_required.0.iter_layertag().next().is_none());
        assert!(abort_disable.0.iter_layertag().next().is_none());
    }
}
