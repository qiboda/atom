#[derive(Default, Clone, Debug)]
pub struct VertexIndices {
    vertex_index: u32,
}

impl VertexIndices {
    pub fn new() -> Self {
        VertexIndices {
            vertex_index: u32::MAX,
        }
    }
}

impl VertexIndices {
    pub fn set_vertex_index(&mut self, x: u32) {
        assert!(x != u32::MAX);
        self.vertex_index = x;
    }

    pub fn set_dir_vertex_index(&mut self, vertex_index: u32) {
        assert!(vertex_index != u32::MAX);
        self.vertex_index = vertex_index;
    }

    pub fn get_dir_vertex_index(&self) -> Option<u32> {
        if self.vertex_index == u32::MAX {
            None
        } else {
            Some(self.vertex_index)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.vertex_index == u32::MAX
    }

    pub fn set_empty(&mut self) {
        self.vertex_index = u32::MAX;
    }
}
