use bevy::{
    asset::{load_internal_asset, Handle},
    prelude::{MaterialPlugin, Plugin, Shader},
};

use crate::shapes::points::material::PointsMaterial;

pub const POINT_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(15532858032624716725);

pub struct PointsPlugin;

impl Plugin for PointsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // TODO: use emmbeded_asset!();
        // embedded_asset!(app, "shaders/point.wgsl");

        load_internal_asset!(
            app,
            POINT_SHADER_HANDLE,
            "shaders/point.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins(MaterialPlugin::<PointsMaterial>::default());
    }
}
