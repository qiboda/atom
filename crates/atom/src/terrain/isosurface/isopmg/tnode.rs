use bevy::prelude::{Vec3, Vec4};

pub struct TNode {
    pub children: [Option<Box<TNode>>; 8],

    pub node: Vec4,

    pub vertices: [Vec4; 8],
    pub edges: [Vec4; 12],
    pub faces: [Vec4; 6],
}

impl TNode {
    pub fn eval(&self, grad: &Vec3, guide: Option<Box<TNode>>) {
        let qef_error = 0.0;

        if (guide.is_none() || guide.unwrap().children[0].is_none()) {
            TNode::vert_node(&mut self.node, grad, &mut qef_error);
        }
    }
}

impl TNode {
    fn vert_node(node: &mut Vec4, grad: Vec3, qef_error: &mut f32) -> _ {
        let qef: [Quadric; 4];

        let mid = 0;

        let plane_norms, plane_points;

        todo!()
    }
}
