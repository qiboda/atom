use bevy_math::{Mat3, Vec3};

/// two vectors cross product, 3x1 * 1x3
pub(crate) fn self_outer_product(v: Vec3) -> Mat3 {
    let a = v.x;
    let b = v.y;
    let c = v.z;

    let mat = Mat3::from_cols(
        Vec3::new(a * a, a * b, a * c),
        Vec3::new(b * a, b * b, b * c),
        Vec3::new(c * a, c * b, c * c),
    );

    mat
}

/// trace is Tr(A) = sigma(i=0..n) A_ii
/// Tr(AB) = Tr(BA) = Tr(BA^T) = Tr(A^TB)
/// Tr(AB) = sigma(i=0..n) sigma(j=0..m) A_ij B_ji
pub(crate) fn trace_of_product(a: Mat3, b: Mat3) -> f32 {
    let mut r = 0.0;
    for i in 0..3 {
        for j in 0..3 {
            r += a.as_ref()[i * 3 + j] * b.as_ref()[j * 3 + i];
        }
    }
    r
}

pub(crate) fn cross_interference_matrix(p: Mat3, q: Mat3) -> Mat3 {
    let mut m = Mat3::default();

    let cxx = p.y_axis.z * q.y_axis.z;
    let cyy = p.x_axis.z * q.x_axis.z;
    let czz = p.x_axis.y * q.x_axis.y;

    m.x_axis.x = p.y_axis.y * q.z_axis.z - cxx - cxx + p.z_axis.z * q.y_axis.y;

    m.y_axis.y = p.x_axis.x * q.z_axis.z - cyy - cyy + p.z_axis.z * q.x_axis.x;

    m.z_axis.z = p.x_axis.x * q.y_axis.y - czz - czz + p.y_axis.y * q.x_axis.x;

    m.x_axis.y = -p.x_axis.y * q.z_axis.z + p.x_axis.z * q.y_axis.z + p.y_axis.z * q.x_axis.z
        - p.z_axis.z * q.x_axis.y;

    m.x_axis.z = p.x_axis.y * q.y_axis.z - p.x_axis.z * q.y_axis.y - p.y_axis.y * q.x_axis.z
        + p.y_axis.z * q.x_axis.y;

    m.y_axis.z = -p.x_axis.x * q.y_axis.z + p.x_axis.y * q.x_axis.z + p.x_axis.z * q.x_axis.y
        - p.y_axis.z * q.x_axis.x;

    m.y_axis.x = m.x_axis.y;

    m.z_axis.x = m.x_axis.z;

    m.z_axis.y = m.y_axis.z;

    m
}

pub(crate) fn first_order_tri_quad(a: Vec3, sigma: Mat3) -> Mat3 {
    let mut m = Mat3::default();

    let xx = a.x * a.x;
    let yy = a.y * a.y;
    let zz = a.z * a.z;
    let xy = a.x * a.y;
    let xz = a.x * a.z;
    let yz = a.y * a.z;

    m.x_axis.x = -sigma.y_axis.y * zz + 2.0 * sigma.y_axis.z * yz - sigma.z_axis.z * yy;

    m.x_axis.y =
        sigma.x_axis.y * zz - sigma.x_axis.z * yz + sigma.y_axis.z * xz + sigma.z_axis.z * xy;

    m.x_axis.z =
        -sigma.x_axis.y * yz + sigma.x_axis.z * yy + sigma.y_axis.y * xz - sigma.y_axis.z * xy;

    m.y_axis.x = m.x_axis.y;

    m.y_axis.y = -sigma.x_axis.x * zz + 2.0 * sigma.x_axis.z * xz - sigma.z_axis.z * xx;

    m.y_axis.z =
        sigma.x_axis.x * yz - sigma.x_axis.y * xz - sigma.x_axis.z * xy + sigma.y_axis.z * xx;

    m.z_axis.x = m.x_axis.z;

    m.z_axis.y = m.y_axis.z;

    m.z_axis.z = -sigma.x_axis.x * yy + 2.0 * sigma.x_axis.y * xy - sigma.y_axis.y * xx;

    m
}

pub(crate) fn cross_product_squared_transpose(v: Vec3) -> Mat3 {
    let a = v[0];
    let b = v[1];
    let c = v[2];
    let a2 = a * a;
    let b2 = b * b;
    let c2 = c * c;

    let mut m: Mat3 = Default::default();

    m.x_axis.x = b2 + c2;
    m.y_axis.y = a2 + c2;
    m.z_axis.z = a2 + b2;

    m.y_axis.x = -a * b;
    m.z_axis.x = -a * c;
    m.z_axis.y = -b * c;

    m.x_axis.y = -a * b;
    m.x_axis.z = -a * c;
    m.y_axis.z = -b * c;

    return m;
}
