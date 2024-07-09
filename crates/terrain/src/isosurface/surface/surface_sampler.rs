use bevy::prelude::Vec3;

use super::shape_surface::ShapeSurface;

pub trait SurfaceSampler {
    fn get_shape_surface(&self) -> &ShapeSurface;

    fn get_value_from_pos(&self, vertex_pos: Vec3) -> f32 {
        self.get_shape_surface().get_value_from_vec(vertex_pos)
    }
}
