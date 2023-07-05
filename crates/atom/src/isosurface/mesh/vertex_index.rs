use bevy::prelude::UVec3;

use crate::isosurface::octree::tables::EdgeDirection;

#[derive(Default, Clone, Debug)]
pub struct VertexIndices {
    vertex_index: UVec3,
}

impl VertexIndices {
    pub fn new() -> Self {
        VertexIndices {
            vertex_index: UVec3::splat(u32::MAX),
        }
    }
}

impl VertexIndices {
    pub fn set_vertex_index(&mut self, x: u32, y: u32, z: u32) {
        assert!(x != u32::MAX && y != u32::MAX && z != u32::MAX);
        self.vertex_index = UVec3::new(x, y, z);
    }

    pub fn set_dir_vertex_index(&mut self, dir: EdgeDirection, vertex_index: u32) {
        assert!(vertex_index != u32::MAX);
        match dir {
            EdgeDirection::XAxis => self.vertex_index.x = vertex_index,
            EdgeDirection::YAxis => self.vertex_index.y = vertex_index,
            EdgeDirection::ZAxis => self.vertex_index.z = vertex_index,
        }
    }

    pub fn get_dir_vertex_index(&self, dir: EdgeDirection) -> Option<u32> {
        let value = match dir {
            EdgeDirection::XAxis => self.vertex_index.x,
            EdgeDirection::YAxis => self.vertex_index.y,
            EdgeDirection::ZAxis => self.vertex_index.z,
        };

        if value == u32::MAX {
            None
        } else {
            Some(value)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.vertex_index.x == u32::MAX
            && self.vertex_index.y == u32::MAX
            && self.vertex_index.z == u32::MAX
    }

    pub fn set_empty(&mut self) {
        self.vertex_index = UVec3::splat(u32::MAX);
    }
}
