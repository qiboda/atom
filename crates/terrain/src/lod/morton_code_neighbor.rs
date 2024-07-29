use bevy::math::UVec3;

use crate::tables::{
    EdgeIndex, FaceIndex, VertexIndex, NEIGHBOR_NODE_IN_EDGE, NEIGHBOR_NODE_IN_VERTEX,
    SUBNODE_IN_EDGE, SUBNODE_IN_FACE,
};

use super::morton_code::MortonCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum MortonCodeError {
    #[error("cell index out of range")]
    CellIndexOutOfRange,
}

pub trait MortonCodeNeighbor {
    fn get_neighbor_face_grid_index(current_xyz: UVec3, face_index: FaceIndex) -> UVec3;
    fn get_neighbor_face_morton_code(
        &self,
        face_index: FaceIndex,
    ) -> Result<MortonCode, MortonCodeError>;
    fn get_neighbor_edge_morton_code(
        &self,
        edge_index: EdgeIndex,
    ) -> Result<MortonCode, MortonCodeError>;
    fn get_neighbor_vertex_morton_code(
        &self,
        vertex_index: VertexIndex,
    ) -> Result<MortonCode, MortonCodeError>;

    fn get_children_morton_code_by_face(&self, face_index: FaceIndex) -> Vec<MortonCode>;
    fn get_children_morton_code_by_edge(&self, edge_index: EdgeIndex) -> Vec<MortonCode>;
    fn get_children_morton_code_by_vertex(&self, vertex_index: VertexIndex) -> Vec<MortonCode>;
}

impl MortonCodeNeighbor for MortonCode {
    fn get_neighbor_face_grid_index(current_xyz: UVec3, face_index: FaceIndex) -> UVec3 {
        match face_index {
            FaceIndex::Left => {
                UVec3::new(current_xyz.x.wrapping_sub(1), current_xyz.y, current_xyz.z)
            }
            FaceIndex::Right => {
                UVec3::new(current_xyz.x.wrapping_add(1), current_xyz.y, current_xyz.z)
            }
            FaceIndex::Bottom => {
                UVec3::new(current_xyz.x, current_xyz.y.wrapping_sub(1), current_xyz.z)
            }
            FaceIndex::Top => {
                UVec3::new(current_xyz.x, current_xyz.y.wrapping_add(1), current_xyz.z)
            }
            FaceIndex::Back => {
                UVec3::new(current_xyz.x, current_xyz.y, current_xyz.z.wrapping_sub(1))
            }
            FaceIndex::Front => {
                UVec3::new(current_xyz.x, current_xyz.y, current_xyz.z.wrapping_add(1))
            }
        }
    }

    fn get_neighbor_face_morton_code(
        &self,
        face_index: FaceIndex,
    ) -> Result<MortonCode, MortonCodeError> {
        let current_xyz = self.decode();
        let neighbor_xyz = MortonCode::get_neighbor_face_grid_index(current_xyz, face_index);
        if neighbor_xyz.x == u32::MAX
            || neighbor_xyz.x >= (1 << self.level())
            || neighbor_xyz.y == u32::MAX
            || neighbor_xyz.y >= (1 << self.level())
            || neighbor_xyz.z == u32::MAX
            || neighbor_xyz.z >= (1 << self.level())
        {
            return Err(MortonCodeError::CellIndexOutOfRange);
        }
        Ok(MortonCode::encode(neighbor_xyz, self.level()))
    }

    fn get_neighbor_edge_morton_code(
        &self,
        edge_index: EdgeIndex,
    ) -> Result<MortonCode, MortonCodeError> {
        let mut current_xyz = self.decode();
        for face_index in NEIGHBOR_NODE_IN_EDGE[edge_index.to_index()] {
            current_xyz = MortonCode::get_neighbor_face_grid_index(current_xyz, face_index);
            if current_xyz.x == u32::MAX
                || current_xyz.x >= (1 << self.level())
                || current_xyz.y == u32::MAX
                || current_xyz.y >= (1 << self.level())
                || current_xyz.z == u32::MAX
                || current_xyz.z >= (1 << self.level())
            {
                return Err(MortonCodeError::CellIndexOutOfRange);
            }
        }
        Ok(MortonCode::encode(current_xyz, self.level()))
    }

    fn get_neighbor_vertex_morton_code(
        &self,
        vertex_index: VertexIndex,
    ) -> Result<MortonCode, MortonCodeError> {
        let mut current_xyz = self.decode();
        for face_index in NEIGHBOR_NODE_IN_VERTEX[vertex_index.to_index()] {
            current_xyz = MortonCode::get_neighbor_face_grid_index(current_xyz, face_index);
            if current_xyz.x == u32::MAX
                || current_xyz.x >= (1 << self.level())
                || current_xyz.y == u32::MAX
                || current_xyz.y >= (1 << self.level())
                || current_xyz.z == u32::MAX
                || current_xyz.z >= (1 << self.level())
            {
                return Err(MortonCodeError::CellIndexOutOfRange);
            }
        }
        Ok(MortonCode::encode(current_xyz, self.level()))
    }

    fn get_children_morton_code_by_face(&self, face_index: FaceIndex) -> Vec<MortonCode> {
        let mut children = vec![];
        let subnodes = SUBNODE_IN_FACE[face_index.to_index()];
        for node_index in subnodes {
            if let Some(child) = self.child(node_index) {
                children.push(child);
            }
        }
        children
    }

    fn get_children_morton_code_by_edge(&self, edge_index: EdgeIndex) -> Vec<MortonCode> {
        let mut children = vec![];
        let subnodes = SUBNODE_IN_EDGE[edge_index.to_index()];
        for node_index in subnodes {
            if let Some(child) = self.child(node_index) {
                children.push(child);
            }
        }
        children
    }

    fn get_children_morton_code_by_vertex(&self, vertex_index: VertexIndex) -> Vec<MortonCode> {
        let mut children = vec![];
        if let Some(child) = self.child(vertex_index) {
            children.push(child);
        }
        children
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::UVec3;

    use crate::{
        lod::morton_code::MortonCode,
        tables::{EdgeIndex, FaceIndex, VertexIndex},
    };

    use super::MortonCodeNeighbor;

    #[test]
    fn test_children_morton_code_by_vertex() {
        let morton_code = MortonCode::encode([0, 0, 0].into(), 0);
        let children = morton_code.get_children_morton_code_by_vertex(VertexIndex::X0Y0Z0);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].level(), 1);
        assert_eq!(children[0].decode(), UVec3::new(0, 0, 0));
    }

    #[test]
    fn test_children_morton_code_by_edge() {
        let morton_code = MortonCode::encode([0, 0, 0].into(), 0);
        let children = morton_code.get_children_morton_code_by_edge(EdgeIndex::XAxisY0Z0);
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].level(), 1);
        assert_eq!(children[1].level(), 1);
        assert_eq!(children[0].decode(), UVec3::new(0, 0, 0));
        assert_eq!(children[1].decode(), UVec3::new(1, 0, 0));
    }

    #[test]
    fn test_hierarchy_morton_code_by_face() {
        let morton_code = MortonCode::encode([0, 2, 0].into(), 3);

        let children = morton_code.get_children_morton_code_by_face(FaceIndex::Left);
        let mut children_vec = vec![];
        for child in children {
            let children = child.get_children_morton_code_by_face(FaceIndex::Left);
            children_vec.extend(children);
        }
        for child in children_vec {
            assert_eq!(child.parent().unwrap().parent().unwrap(), morton_code);
        }
    }

    #[test]
    fn test_children_morton_code_by_face() {
        let morton_code = MortonCode::encode([0, 0, 0].into(), 0);
        let children = morton_code.get_children_morton_code_by_face(FaceIndex::Left);
        assert_eq!(children.len(), 4);
        assert_eq!(children[0].level(), 1);
        assert_eq!(children[1].level(), 1);
        assert_eq!(children[2].level(), 1);
        assert_eq!(children[3].level(), 1);
        assert_eq!(children[0].decode(), UVec3::new(0, 0, 0));
        assert_eq!(children[1].decode(), UVec3::new(0, 1, 0));
        assert_eq!(children[2].decode(), UVec3::new(0, 0, 1));
        assert_eq!(children[3].decode(), UVec3::new(0, 1, 1));
    }

    #[test]
    fn test_neighbor_face_twin() {
        let morton_code = MortonCode::encode([0, 0, 0].into(), 3);
        assert_eq!(
            morton_code,
            morton_code
                .get_neighbor_face_morton_code(FaceIndex::Right)
                .unwrap()
                .get_neighbor_face_morton_code(FaceIndex::Left)
                .unwrap()
        );
        assert_eq!(
            morton_code,
            morton_code
                .get_neighbor_face_morton_code(FaceIndex::Top)
                .unwrap()
                .get_neighbor_face_morton_code(FaceIndex::Bottom)
                .unwrap()
        );
    }

    #[test]
    fn test_neighbor_face_grid_index() {
        let grid_index =
            MortonCode::get_neighbor_face_grid_index(UVec3::new(0, 0, 0), FaceIndex::Left);
        assert_eq!(grid_index, UVec3::new(u32::MAX, 0, 0));
    }

    #[test]
    fn test_neighbor_morton_code_by_face() {
        let morton_code = MortonCode::encode([0, 0, 0].into(), 0);
        let children = morton_code.get_neighbor_face_morton_code(FaceIndex::Left);
        assert!(children.is_err());

        let morton_code = MortonCode::encode([1, 0, 0].into(), 1);
        let neighbor_code = morton_code
            .get_neighbor_face_morton_code(FaceIndex::Left)
            .unwrap();
        assert_eq!(neighbor_code, MortonCode::encode([0, 0, 0].into(), 1));

        let morton_code = MortonCode::encode([2, 0, 0].into(), 2);
        let neighbor_code = morton_code
            .get_neighbor_face_morton_code(FaceIndex::Left)
            .unwrap();
        assert_eq!(neighbor_code, MortonCode::encode([1, 0, 0].into(), 2));
    }

    #[test]
    fn test_neighbor_morton_code_by_edge() {
        let morton_code = MortonCode::encode([0, 0, 0].into(), 0);
        let children = morton_code.get_neighbor_edge_morton_code(EdgeIndex::XAxisY0Z0);
        assert!(children.is_err());

        let morton_code = MortonCode::encode([0, 0, 0].into(), 1);
        let children = morton_code.get_neighbor_edge_morton_code(EdgeIndex::YAxisX1Z0);
        assert!(children.is_err());

        let morton_code = MortonCode::encode([3, 4, 6].into(), 4);
        let children = morton_code
            .get_neighbor_edge_morton_code(EdgeIndex::YAxisX1Z0)
            .unwrap();
        assert_eq!(children.level(), 4);
        assert_eq!(children.decode(), UVec3::new(4, 4, 5));

        let morton_code = MortonCode::encode([3, 4, 6].into(), 4);
        let nei = morton_code
            .get_neighbor_edge_morton_code(EdgeIndex::YAxisX0Z0)
            .unwrap();
        assert_eq!(nei.level(), 4);
        assert_eq!(nei.decode(), UVec3::new(2, 4, 5));

        let children = nei.get_children_morton_code_by_edge(EdgeIndex::YAxisX0Z0);
        assert_eq!(children[0].decode(), UVec3::new(4, 8, 10));
        assert_eq!(children[1].decode(), UVec3::new(4, 9, 10));
    }

    #[test]
    fn test_neighbor_morton_code_by_vertex() {
        let morton_code = MortonCode::encode([0, 0, 0].into(), 0);
        let children = morton_code.get_neighbor_vertex_morton_code(VertexIndex::X0Y0Z0);
        assert!(children.is_err());

        let morton_code = MortonCode::encode([0, 0, 1].into(), 1);
        let children = morton_code.get_neighbor_vertex_morton_code(VertexIndex::X1Y1Z1);
        assert!(children.is_err());

        let morton_code = MortonCode::encode([3, 4, 6].into(), 4);
        let children = morton_code
            .get_neighbor_vertex_morton_code(VertexIndex::X1Y0Z0)
            .unwrap();
        assert_eq!(children.level(), 4);
        assert_eq!(children.decode(), UVec3::new(4, 3, 5));
    }

    #[test]
    fn test_neighbor_depth_and_grid_size() {
        let morton_code = MortonCode::encode([128, 128, 129].into(), 8);

        println!("morton node: {:?}", morton_code.decode());
        let front_code = morton_code
            .get_neighbor_face_morton_code(FaceIndex::Front)
            .unwrap();
        println!("front node: {:?}", front_code.decode());
        let front_children = front_code.get_children_morton_code_by_face(FaceIndex::Back);
        assert_eq!(front_children.len(), 4);
        for child in front_children {
            println!("front node: {:?}", child.decode());
        }
        let right_code = morton_code
            .get_neighbor_face_morton_code(FaceIndex::Right)
            .unwrap();
        println!("right node: {:?}", right_code.decode());
        let right_children = right_code.get_children_morton_code_by_face(FaceIndex::Left);
        assert_eq!(right_children.len(), 4);
        for child in right_children {
            println!("right node: {:?}", child.decode());
        }
    }
}
