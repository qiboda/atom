use bevy::math::Vec3;

use super::CSGNode;

#[derive(Debug)]
pub struct CSGMax {
    pub left: Box<dyn CSGNode>,
    pub right: Box<dyn CSGNode>,
}

/// 合并, 融合, 并集
impl CSGNode for CSGMax {
    fn eval(&self, point: &Vec3, value: &mut f32) {
        let mut left_value = 0.0;
        self.left.eval(point, &mut left_value);

        let mut right_value = 0.0;
        self.right.eval(point, &mut right_value);

        if left_value > right_value {
            *value = left_value;
        } else {
            *value = right_value;
        }
    }
}

// 相交的部分, 交集
#[derive(Debug)]
pub struct CSGMin {
    pub left: Box<dyn CSGNode>,
    pub right: Box<dyn CSGNode>,
}

impl CSGNode for CSGMin {
    fn eval(&self, point: &Vec3, value: &mut f32) {
        let mut left_value = 0.0;
        self.left.eval(point, &mut left_value);

        let mut right_value = 0.0;
        self.right.eval(point, &mut right_value);

        if left_value < right_value {
            *value = left_value;
        } else {
            *value = right_value;
        }
    }
}

// 不是差集，是相减
#[derive(Debug)]
pub struct CSGMinus {
    pub left: Box<dyn CSGNode>,
    pub right: Box<dyn CSGNode>,
}

impl CSGNode for CSGMinus {
    fn eval(&self, point: &Vec3, value: &mut f32) {
        let mut left_value = 0.0;
        self.left.eval(point, &mut left_value);

        let mut right_value = 0.0;
        self.right.eval(point, &mut right_value);

        if left_value > right_value {
            *value = left_value;
        } else {
            *value = -right_value;
        }
    }
}

// 差集
#[derive(Debug)]
pub struct CSGDiff {
    pub left: Box<dyn CSGNode>,
    pub right: Box<dyn CSGNode>,
}

impl CSGNode for CSGDiff {
    fn eval(&self, point: &Vec3, value: &mut f32) {
        let mut left_value = 0.0;
        self.left.eval(point, &mut left_value);

        let mut right_value = 0.0;
        self.right.eval(point, &mut right_value);

        if right_value < 0.0 {
            *value = -right_value;
        } else {
            *value = left_value;
        }
    }
}

// 取反，将外部变为内部，内部变为外部
#[derive(Debug)]
pub struct CSGNeg {
    pub node: Box<dyn CSGNode>,
}

impl CSGNode for CSGNeg {
    fn eval(&self, point: &Vec3, value: &mut f32) {
        let mut node_value = 0.0;
        self.node.eval(point, &mut node_value);

        *value = -node_value;
    }
}
