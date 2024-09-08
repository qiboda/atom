use bevy::prelude::*;

#[derive(Bundle)]
pub struct ClientSpatialBundle {
    pub global_transform: GlobalTransform,

    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl Default for ClientSpatialBundle {
    fn default() -> Self {
        Self {
            global_transform: Default::default(),
            inherited_visibility: InheritedVisibility::VISIBLE,
            view_visibility: Default::default(),
        }
    }
}

#[derive(Bundle, Default)]
pub struct ServerSpatialBundle {
    pub transform: Transform,

    pub visibility: Visibility,
}
