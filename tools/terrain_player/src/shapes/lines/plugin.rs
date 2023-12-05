use bevy::{prelude::*, asset::load_internal_asset};

use crate::shapes::lines::material::LineMaterial;

pub const LINE_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(280840713593477678860567649031760994175);

struct LinesPluigns;

impl Plugin for LinesPluigns {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            LINE_SHADER_HANDLE,
            "shaders/line.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins(MaterialPlugin::<LineMaterial>::default());
    }
}
