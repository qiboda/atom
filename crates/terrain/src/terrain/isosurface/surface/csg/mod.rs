use bevy::prelude::Vec3;

pub trait CSGNode {
    fn eval(&self, point: &Vec3, value: &mut f32, grad: &mut Vec3);
}

pub struct CSGMax {
    pub left: Box<dyn CSGNode>,
    pub right: Box<dyn CSGNode>,
}

impl CSGNode for CSGMax {
    fn eval(&self, point: &Vec3, value: &mut f32, grad: &mut Vec3) {
        let mut left_value = 0.0;
        let mut left_grad = Vec3::ZERO;
        self.left.eval(point, &mut left_value, &mut left_grad);

        let mut right_value = 0.0;
        let mut right_grad = Vec3::ZERO;
        self.right.eval(point, &mut right_value, &mut right_grad);

        if left_value > right_value {
            *value = left_value;
            *grad = left_grad;
        } else {
            *value = right_value;
            *grad = right_grad;
        }
    }
}

pub struct CSGMin {
    pub left: Box<dyn CSGNode>,
    pub right: Box<dyn CSGNode>,
}

impl CSGNode for CSGMin {
    fn eval(&self, point: &Vec3, value: &mut f32, grad: &mut Vec3) {
        let mut left_value = 0.0;
        let mut left_grad = Vec3::ZERO;
        self.left.eval(point, &mut left_value, &mut left_grad);

        let mut right_value = 0.0;
        let mut right_grad = Vec3::ZERO;
        self.right.eval(point, &mut right_value, &mut right_grad);

        if left_value < right_value {
            *value = left_value;
            *grad = left_grad;
        } else {
            *value = right_value;
            *grad = right_grad;
        }
    }
}

pub struct CSGDiff {
    pub left: Box<dyn CSGNode>,
    pub right: Box<dyn CSGNode>,
}

impl CSGNode for CSGDiff {
    fn eval(&self, point: &Vec3, value: &mut f32, grad: &mut Vec3) {
        let mut left_value = 0.0;
        let mut left_grad = Vec3::ZERO;
        self.left.eval(point, &mut left_value, &mut left_grad);

        let mut right_value = 0.0;
        let mut right_grad = Vec3::ZERO;
        self.right.eval(point, &mut right_value, &mut right_grad);

        if left_value > right_value {
            *value = left_value;
            *grad = left_grad;
        } else {
            *value = -right_value;
            *grad = -right_grad;
        }
    }
}

pub struct CSGNeg {
    pub node: Box<dyn CSGNode>,
}

impl CSGNode for CSGNeg {
    fn eval(&self, point: &Vec3, value: &mut f32, grad: &mut Vec3) {
        let mut node_value = 0.0;
        let mut node_grad = Vec3::ZERO;
        self.node.eval(point, &mut node_value, &mut node_grad);

        *value = -node_value;
        *grad = -node_grad;
    }
}

pub struct CSGPlane {
    pub position: Vec3,
    pub normal: Vec3,
}

impl CSGNode for CSGPlane {
    fn eval(&self, point: &Vec3, value: &mut f32, grad: &mut Vec3) {
        let diff = *point - self.position;
        *value = diff.dot(self.normal);
        *grad = self.normal;
    }
}

pub struct CSGSphere {
    pub position: Vec3,
    pub radius: f32,
}

impl CSGNode for CSGSphere {
    fn eval(&self, point: &Vec3, value: &mut f32, grad: &mut Vec3) {
        let diff = *point - self.position;
        *value = diff.length() - self.radius;
        *grad = diff.normalize();
    }
}

pub struct CSGCylinder {
    pub position: Vec3,
    pub direction: Vec3,
    pub radius: f32,
}

impl CSGNode for CSGCylinder {
    fn eval(&self, point: &Vec3, value: &mut f32, grad: &mut Vec3) {
        let diff = *point - self.position;
        let d = diff.dot(self.direction);
        let p = self.position + self.direction * d;
        let r = (p - *point).length() - self.radius;

        *value = r;
        *grad = (p - *point).normalize();
    }
}

pub struct CSGTorus {
    pub position: Vec3,
    pub radius: f32,
    pub thickness: f32,
}

impl CSGNode for CSGTorus {
    fn eval(&self, point: &Vec3, value: &mut f32, grad: &mut Vec3) {
        let diff = *point - self.position;
        let x = diff.x;
        let z = diff.z;
        let y = diff.y;

        let r = (x * x + z * z).sqrt() - self.radius;
        let d = (r * r + y * y).sqrt() - self.thickness;

        *value = d;
        *grad = Vec3::new(
            4.0 * x * d,
            2.0 * y * d,
            4.0 * z * d + 2.0 * r * (r - self.thickness),
        )
        .normalize();
    }
}

#[allow(dead_code)]
fn build_shape() -> Box<CSGMin> {
    let shift = Vec3::new(0.062151346, 0.0725234, 0.0412);

    let one_box = Box::new(CSGMin {
        left: Box::new(CSGMin {
            left: Box::new(CSGMin {
                left: Box::new(CSGPlane {
                    position: Vec3::new(0.3, 0.3, 0.3) + shift,
                    normal: Vec3::new(1.0, 0.0, 0.0),
                }),
                right: Box::new(CSGPlane {
                    position: Vec3::new(0.3, 0.3, 0.3) + shift,
                    normal: Vec3::new(0.0, 1.0, 0.0),
                }),
            }),
            right: Box::new(CSGMin {
                left: Box::new(CSGPlane {
                    position: Vec3::new(0.3, 0.3, 0.3) + shift,
                    normal: Vec3::new(0.0, 0.0, 1.0),
                }),
                right: Box::new(CSGPlane {
                    position: Vec3::new(0.7, 0.7, 0.7) + shift,
                    normal: Vec3::new(-1.0, 0.0, 0.0),
                }),
            }),
        }),
        right: Box::new(CSGMin {
            left: Box::new(CSGPlane {
                position: Vec3::new(0.7, 0.7, 0.7) + shift,
                normal: Vec3::new(0.0, -1.0, 0.0),
            }),
            right: Box::new(CSGPlane {
                position: Vec3::new(0.7, 0.7, 0.7) + shift,
                normal: Vec3::new(0.0, 0.0, -1.0),
            }),
        }),
    });

    let n = Box::new(CSGMin {
        left: Box::new(CSGNeg {
            node: Box::new(CSGCylinder {
                position: Vec3::new(0.5, 0.5, 0.5) + shift,
                direction: Vec3::new(1.0, 0.0, 0.0),
                radius: 0.15,
            }),
        }),
        right: Box::new(CSGMin {
            left: Box::new(CSGNeg {
                node: Box::new(CSGCylinder {
                    position: Vec3::new(0.5, 0.5, 0.5) + shift,
                    direction: Vec3::new(0.0, 1.0, 0.0),
                    radius: 0.15,
                }),
            }),
            right: Box::new(CSGMax {
                left: one_box,
                right: Box::new(CSGCylinder {
                    position: Vec3::new(0.5, 0.5, 0.5) + shift,
                    direction: Vec3::new(0.0, 0.0, 1.0),
                    radius: 0.15,
                }),
            }),
        }),
    });

    

    Box::new(CSGMin {
        left: n,
        right: Box::new(CSGMin {
            left: Box::new(CSGPlane {
                position: Vec3::new(0.0, 0.0, 0.9) + shift,
                normal: Vec3::new(0.0, 0.0, -1.0),
            }),
            right: Box::new(CSGPlane {
                position: Vec3::new(0.0, 0.0, 0.1) + shift,
                normal: Vec3::new(0.0, 0.0, 1.0),
            }),
        }),
    })
}
