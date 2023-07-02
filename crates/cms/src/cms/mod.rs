pub mod bundle;
pub mod cms;
pub mod seg_component;
pub mod tessellation;

use bevy::prelude::*;

use crate::{octree::OctreePlugin, surface::shape_surface::ShapeSurface};

#[derive(Default, Debug)]
pub struct CMSPlugin;

/// todo: 坐标变换，从密度函数的坐标系到采样坐标系
impl Plugin for CMSPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ShapeSurface::default());
        app.add_plugin(OctreePlugin);
    }
}
