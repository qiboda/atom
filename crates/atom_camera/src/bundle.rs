use bevy::prelude::*;

#[derive(Component, Debug, Default)]
pub struct ActiveCamera {
    pub mode: CameraMode,
    pub last_mode_transform: Transform,
}

#[derive(Debug, Default)]
pub enum CameraMode {
    #[default]
    Player,
    Debug,
}

#[derive(Bundle, Default)]
pub struct ActiveCameraBundle {
    camera_bundle: Camera3dBundle,
    active_camera: ActiveCamera,
}
