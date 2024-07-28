pub mod csg_noise;
pub mod csg_operators;
pub mod csg_shapes;
pub mod falloff_map;
pub mod noise_cache;
pub mod aworley;
pub mod arc_noise;

use std::fmt::Debug;

use bevy::prelude::Vec3;

pub trait CSGNode: Debug + Send + Sync {
    fn eval(&self, point: &Vec3, value: &mut f32);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CSGOperation {
    Union,
    Intersection,
    Difference,
}

fn create_internal_node(
    operation: CSGOperation,
    left: Box<dyn CSGNode>,
    right: Box<dyn CSGNode>,
) -> Box<dyn CSGNode> {
    match operation {
        CSGOperation::Union => Box::new(csg_operators::CSGMax { left, right }),
        CSGOperation::Intersection => Box::new(csg_operators::CSGMin { left, right }),
        CSGOperation::Difference => Box::new(csg_operators::CSGDiff { left, right }),
    }
}

// 添加删除类型的几何体。
// 添加增加类型的集合体。
#[allow(dead_code)]
pub fn apply_csg_operation(
    root: Box<dyn CSGNode>,
    new_node: Box<dyn CSGNode>,
    operation: CSGOperation,
) -> Box<dyn CSGNode> {
    create_internal_node(operation, root, new_node)
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec3;

    use crate::isosurface::surface::shape_surface::ShapeSurface;

    use super::{
        csg_shapes::{CSGCube, CSGPanel},
        CSGOperation,
    };

    #[test]
    fn test_csg() {
        let mut shape_surface = ShapeSurface::new(Box::new(CSGPanel {
            location: Vec3::ZERO,
            normal: Vec3::Y,
            height: 0.0,
        }));

        assert_eq!(0.0, shape_surface.get_value(5.0, 0.0, 5.0));
        assert_eq!(-2.0, shape_surface.get_value(5.0, -2.0, 5.0));
        assert_eq!(-2.0, shape_surface.get_value(3.0, -2.0, 7.0));
        assert_eq!(0.0, shape_surface.get_value(0.0, -0.0, 0.0));
        assert_eq!(1.0, shape_surface.get_value(0.0, 1.0, 0.0));
        assert_eq!(-1.0, shape_surface.get_value(0.0, -1.0, 0.0));

        shape_surface.apply_csg_operation(
            Box::new(CSGCube {
                location: Vec3::new(5.0, 0.0, 5.0),
                half_size: Vec3::splat(3.0),
            }),
            CSGOperation::Difference,
        );

        assert_eq!(3.0, shape_surface.get_value(5.0, 0.0, 5.0));
        assert_eq!(1.0, shape_surface.get_value(5.0, -2.0, 5.0));
        assert_eq!(1.0, shape_surface.get_value(3.0, -2.0, 7.0));
        assert_eq!(-0.0, shape_surface.get_value(0.0, -0.0, 0.0));
        assert_eq!(-1.0, shape_surface.get_value(0.0, 1.0, 0.0));
        assert_eq!(1.0, shape_surface.get_value(0.0, -1.0, 0.0));
    }
}
