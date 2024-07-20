use bevy::prelude::Component;

#[derive(Debug, Default, Component)]
pub struct ItemCd {
    pub cd: f32,
}

#[derive(Debug, Default, Component)]
pub struct ItemStack {
    pub stack: i32,
}

#[derive(Debug, Default, Component)]
pub struct ItemCount {
    pub count: i32,
}
