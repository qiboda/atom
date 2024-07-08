//! # Atom Camera
//! 摄像机管理插件
//!
//! ## 功能
//! 切换摄像机。(暂不支持)
//! 控制摄像机的旋转，移动, 与跟踪对象的距离等。
pub mod bundle;
pub mod setting;

use atom_utils::follow::TransformFollowPlugin;
// TODO: format to reorder import
use bevy::prelude::*;
use leafwing_input_manager::plugin::InputManagerPlugin;
use setting::{CameraAction, CameraSetting};
use settings::SettingPlugin;

#[derive(Debug, Default)]
pub struct CameraManagerPlugin;

impl Plugin for CameraManagerPlugin {
    fn build(&self, app: &mut App) {
        assert!(app.is_plugin_added::<TransformFollowPlugin>());

        app.add_plugins(InputManagerPlugin::<CameraAction>::default())
            .add_plugins(SettingPlugin::<CameraSetting>::default())
            .insert_resource(CameraTracker::new())
            .add_systems(Update, setting::zoom_camera);
    }
}

#[derive(Resource, Debug, Default)]
pub struct CameraTracker {
    main_camera: Option<Entity>,
}

impl CameraTracker {
    pub fn new() -> Self {
        Self { main_camera: None }
    }

    pub fn set_main_camera(&mut self, camera: Entity) {
        self.main_camera = Some(camera);
    }

    pub fn get_main_camera(&self) -> Option<Entity> {
        self.main_camera
    }
}
