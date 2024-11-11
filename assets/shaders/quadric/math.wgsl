#define_import_path quadric_math

/// two vectors cross product, 3x1 * 1x3
fn self_outer_product(v: vec3f) -> mat3x3f {
    let a = v.x;
    let b = v.y;
    let c = v.z;

    return mat3x3f(
        vec3f(a * a, a * b, a * c),
        vec3f(b * a, b * b, b * c),
        vec3f(c * a, c * b, c * c),
    );
}

/// trace is Tr(A) = sigma(i=0..n) A_ii
/// Tr(AB) = Tr(BA) = Tr(BA^T) = Tr(A^TB)
/// Tr(AB) = sigma(i=0..n) sigma(j=0..m) A_ij B_ji
fn trace_of_product(a: ptr<function, mat3x3f>, b: ptr<function, mat3x3f>) -> f32 {
    var r = 0.0;
    for (var i = 0u; i < 3u; i++) {
        for (var j = 0u; j < 3u; j++) {
            r += (*a)[i][j] * (*b)[j][i];
        }
    }
    return r;
}

/// TODO: to understand
/// p and q are 3x3 matrices
/// and use p and q 2x2 matrices interference determinant to as interference value at non 2x2
/// matrix row and column
/// such as : m00 = (p11 * q22 - p12 * q21) + (p21 * q12 - p22 * q11)
fn cross_interference_matrix(p: mat3x3f, q: mat3x3f) -> mat3x3f {
    var m = mat3x3f();

    let cxx = p[1].z * q[1].z;
    let cyy = p[0].z * q[0].z;
    let czz = p[0].y * q[0].y;

    m[0].x = p[1].y * q[2].z - cxx - cxx + p[2].z * q[1].y;

    m[1].y = p[0].x * q[2].z - cyy - cyy + p[2].z * q[0].x;

    m[2].z = p[0].x * q[1].y - czz - czz + p[1].y * q[0].x;

    m[0].y = -p[0].y * q[2].z + p[0].z * q[1].z + p[1].z * q[0].z - p[2].z * q[0].y;

    m[0].z = p[0].y * q[1].z - p[0].z * q[1].y - p[1].y * q[0].z + p[1].z * q[0].y;

    m[1].z = -p[0].x * q[1].z + p[0].y * q[0].z + p[0].z * q[0].y - p[1].z * q[0].x;

    m[1].x = m[0].y;

    m[2].x = m[0].z;

    m[2].y = m[1].z;

    return m;
}

fn first_order_tri_quad(a: vec3f, sigma: mat3x3f) -> mat3x3f {
    var m = mat3x3f();

    let xx = a.x * a.x;
    let yy = a.y * a.y;
    let zz = a.z * a.z;
    let xy = a.x * a.y;
    let xz = a.x * a.z;
    let yz = a.y * a.z;

    m[0].x = -sigma[1].y * zz + 2.0 * sigma[1].z * yz - sigma[2].z * yy;

    m[0].y = sigma[0].y * zz - sigma[0].z * yz + sigma[1].z * xz + sigma[2].z * xy;

    m[0].z = -sigma[0].y * yz + sigma[0].z * yy + sigma[1].y * xz - sigma[1].z * xy;

    m[1].x = m[0].y;

    m[1].y = -sigma[0].x * zz + 2.0 * sigma[0].z * xz - sigma[2].z * xx;

    m[1].z = sigma[0].x * yz - sigma[0].y * xz - sigma[0].z * xy + sigma[1].z * xx;

    m[2].x = m[0].z;

    m[2].y = m[1].z;

    m[2].z = -sigma[0].x * yy + 2.0 * sigma[0].y * xy - sigma[1].y * xx;

    return m;
}

/// v => [v]x => [v]x * [v]x^T
/// [v]x is skwy-symmetric matrix
fn cross_product_squared_transpose(v: vec3f) -> mat3x3f {
    let a = v[0];
    let b = v[1];
    let c = v[2];
    let a2 = a * a;
    let b2 = b * b;
    let c2 = c * c;

    var m = mat3x3f();

    m[0][0] = b2 + c2;
    m[1][1] = a2 + c2;
    m[2][2] = a2 + b2;

    m[1][0] = -a * b;
    m[2][0] = -a * c;
    m[2][1] = -b * c;

    m[0][1] = -a * b;
    m[0][2] = -a * c;
    m[1][2] = -b * c;

    return m;
}
