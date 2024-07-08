use bevy::prelude::*;

#[derive(Debug, Component, Default, Copy, Clone, PartialEq, Eq)]
pub struct TranslationLockedFreedom {
    pub locked_x: bool,
    pub locked_y: bool,
    pub locked_z: bool,
}

impl TranslationLockedFreedom {
    pub fn new(locked_x: bool, locked_y: bool, locked_z: bool) -> Self {
        Self {
            locked_x,
            locked_y,
            locked_z,
        }
    }
}

#[derive(Debug, Component, Default, Copy, Clone, PartialEq, Eq)]
pub struct RotationLockedFreedom {
    pub locked_pitch: bool,
    pub locked_yaw: bool,
    pub locked_roll: bool,
}

impl RotationLockedFreedom {
    pub fn new(locked_pitch: bool, locked_yaw: bool, locked_roll: bool) -> Self {
        Self {
            locked_pitch,
            locked_yaw,
            locked_roll,
        }
    }
}

#[derive(Debug, Component, Default, Copy, Clone, PartialEq, Eq)]
pub struct ScaleLockedFreedom {
    pub locked_x: bool,
    pub locked_y: bool,
    pub locked_z: bool,
}

impl ScaleLockedFreedom {
    pub fn new(locked_x: bool, locked_y: bool, locked_z: bool) -> Self {
        Self {
            locked_x,
            locked_y,
            locked_z,
        }
    }
}

#[derive(Debug, Component, Default, Copy, Clone, PartialEq, Eq)]
pub struct TransformLockedFreedom {
    pub locked_translation: Option<TranslationLockedFreedom>,
    pub locked_rotation: Option<RotationLockedFreedom>,
    pub locked_scale: Option<ScaleLockedFreedom>,
}

#[derive(Debug, Component, Default, Copy, Clone, PartialEq, Eq)]
pub enum TransformFreedom {
    #[default]
    None,
    Lock(TransformLockedFreedom),
}
