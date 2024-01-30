use super::address::VertexAddress;

#[allow(dead_code)]
#[derive(Debug)]
struct Edge {
    pub left_vertex_address: VertexAddress,
    pub right_vertex_address: VertexAddress,
}
