use std::ops::DerefMut;

use atom_utils::input::DefaultInputMap;
use bevy::prelude::*;
use bevy::reflect::Reflect;
use leafwing_input_manager::{
    action_state::ActionState, input_map::InputMap, user_input::MouseScrollAxis, Actionlike,
};
use serde::{Deserialize, Serialize};
use settings_derive::Setting;

use crate::CameraTracker;

#[derive(Setting, Resource, Asset, Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct CameraSetting {
    pub camera_input_map: InputMap<CameraAction>,
    pub camera_zoom_rate: f32,
}

impl Default for CameraSetting {
    fn default() -> Self {
        Self {
            camera_input_map: CameraAction::default_input_map(),
            camera_zoom_rate: 0.05,
        }
    }
}

#[derive(Actionlike, Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum CameraAction {
    // 鼠标按住不松，拖动鼠标
    // Pan,
    // 鼠标滚轮。
    Zoom,
    // rotate around y axis
    LeftRotateY,
    RightRotateY,
}

impl DefaultInputMap<CameraAction> for CameraAction {
    fn default_input_map() -> InputMap<CameraAction> {
        let mut input_map = InputMap::default();

        // input_map.insert(CameraAction::Pan, MouseMove::default());
        input_map.insert(CameraAction::Zoom, MouseScrollAxis::Y);
        input_map.insert(CameraAction::LeftRotateY, KeyCode::KeyQ);
        input_map.insert(CameraAction::RightRotateY, KeyCode::KeyE);

        input_map
    }
}

pub fn zoom_camera(
    action_query: Query<&ActionState<CameraAction>>,
    mut camera_query: Query<&mut Projection, With<Camera3d>>,
    camera_tracker: Res<CameraTracker>,
    camera_setting: Res<CameraSetting>,
) {
    if let Ok(action_state) = action_query.get_single() {
        if let Some(Ok(mut camera_projection)) = camera_tracker
            .get_main_camera()
            .map(|camera| camera_query.get_mut(camera))
        {
            let zoom_delta = action_state.value(&CameraAction::Zoom);

            match camera_projection.deref_mut() {
                Projection::Perspective(perspective) => {
                    perspective.fov *= 1. - zoom_delta * camera_setting.camera_zoom_rate;
                }
                Projection::Orthographic(_) => {
                    panic!("not support Orthographic camera yet")
                }
            }
        }
    }
}
