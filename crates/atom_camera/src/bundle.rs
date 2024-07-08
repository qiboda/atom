use bevy::prelude::*;

#[derive(Component, Debug, Default)]
pub enum CameraMode {
    #[default]
    Player,
    Debug,
}

#[derive(Bundle, Default)]
pub struct ActiveCameraBundle {
    camera_bundle: Camera3dBundle,
    camera_mode: CameraMode,
}
