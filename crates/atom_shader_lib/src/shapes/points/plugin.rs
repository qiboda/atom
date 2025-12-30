use bevy::{
    asset::{Handle, load_internal_asset, uuid_handle},
    prelude::{MaterialPlugin, Plugin, Shader},
};

use crate::shapes::points::material::PointsMaterial;

pub const POINT_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("46ed26a8-5848-452a-a12f-bb719fbf4c0d");

pub struct PointsPlugin;

impl Plugin for PointsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        load_internal_asset!(
            app,
            POINT_SHADER_HANDLE,
            "shaders/point.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins(MaterialPlugin::<PointsMaterial>::default());
    }
}
