//! 远程截图支持。
//!
//! 通过 BRP `world.spawn_entity` 传入 `TakeScreenshot` 组件触发截图。
//! Agent 调用：
//! ```ts
//! await brp('world.spawn_entity', {
//!   components: {
//!     'atom_terrain::screenshot::TakeScreenshot': {},
//!   },
//! });
//! ```

use bevy::prelude::*;
use bevy::render::view::window::screenshot::{Screenshot, save_to_disk};

/// 标记组件：spawn 此组件即可触发一帧截图。
/// 需通过 `app.register_type::<TakeScreenshot>()` 注册使 BRP 可访问。
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct TakeScreenshot;

/// 每帧检查新 spawn 的 TakeScreenshot 实体，触发截图后自动清空。
pub fn screenshot_trigger_system(
    mut commands: Commands,
    trigger_query: Query<Entity, With<TakeScreenshot>>,
) {
    for entity in &trigger_query {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("SystemTime before UNIX_EPOCH — clock is wrong")
            .as_millis();
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(format!("screenshots/terrain-{stamp}.png")));
        commands.entity(entity).despawn();
    }
}
