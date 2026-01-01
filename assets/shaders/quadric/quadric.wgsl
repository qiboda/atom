#define_import_path quadric

#import quadric_math::{self_outer_product, trace_of_product, cross_interference_matrix, first_order_tri_quad, cross_product_squared_transpose}

//! 论文是下面这个
//! Computer Graphics Forum - 2020 - Trettner - Fast and Robust QEF Minimization using Probabilistic Quadrics

// E[x] = x^T * A^T * A * x −2 * x^T * A^T * b + b^T * b
// and A = n, b = p * n, x是未知数
// Quadric中的a为A^T A, b为A^T b, c为b^T b
struct Quadric {
    /// a is a symmetric 3x3 matrix
    a00f: f32,
    a01f: f32,
    a02f: f32,
    a11f: f32,
    a12f: f32,
    a22f: f32,

    b0f: f32,
    b1f: f32,
    b2f: f32,

    c: f32,
}

fn quadric_default() -> Quadric {
    return Quadric (
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,

        0.0,
        0.0,
        0.0,

        0.0,
    );
}

fn quadric_from_coefficients_matrix(a: mat3x3f, b: vec3<f32>, c: f32) -> Quadric {
    return Quadric(
        a[0].x,
        a[0].y,
        a[0].z,
        a[1].y,
        a[1].z,
        a[2].z,
        b[0],
        b[1],
        b[2],
        c,
    );
}

    /// 生成一个点的quadric。
fn point_quadric(p: vec3f) -> Quadric {
    return quadric_from_coefficients_matrix(mat3x3f(vec3f(1.0, 0.0, 0.0), vec3f(0.0, 1.0, 0.0), vec3f(0.0, 0.0, 1.0)), p, 0.0);
}

    /// 生成一个平面的quadric，这个平面的法向量是n，平面上的一个点是p
    /// 是一个经典版本的quadric，不考虑不确定性
fn plane_quadric(p: vec3f, n: vec3f) -> Quadric {
    let d = dot(p, n);
    return quadric_from_coefficients_matrix(self_outer_product(n), n * d, d * d);
}

    /// 生成一个平面的quadric，这个平面的法向量是mean_n，平面上的一个点是mean_p
    /// stddev_p和stddev_n分别表示mean_p和mean_n的标准差, stddev_p和stddev_n是各向同性的。
fn probabilistic_plane_quadric(
    mean_p: vec3f,
    mean_n: vec3f,
    stddev_p: f32,
    stddev_n: f32,
) -> Quadric {
    let p = mean_p;
    let n = mean_n;

    let sn2 = stddev_n * stddev_n;
    let sp2 = stddev_p * stddev_p;

    let d = dot(p, n);

    var a = self_outer_product(n);

    a[0].x += sn2;
    a[1].y += sn2;
    a[2].z += sn2;

    let b = n * d + p * sn2;
    let c = d * d + sn2 * dot(p, p) + sp2 * dot(n, n) + 3.0 * sp2 * sn2;

    return quadric_from_coefficients_matrix(a, b, c);
}

    /// sigma => covariance matrix
    /// 生成一个平面的quadric，这个平面的法向量是mean_n，平面上的一个点是mean_p
    /// sigma_p和sigma_n分别表示mean_p和mean_n的协方差矩阵, sigma_p和sigma_n是各向异性的。
fn probabilistic_plane_quadric_sigma(
    mean_p: vec3f,
    mean_n: vec3f,
    sigma_p: mat3x3f,
    sigma_n: mat3x3f,
) -> Quadric {
    let p = mean_p;
    let d = dot(p, mean_n);

    let a = self_outer_product(mean_n) + sigma_n;

    let b = mean_n * d + sigma_n * p;
    let c = d * d + dot(p, sigma_n * p) + dot(mean_n, sigma_p * mean_n) + trace_of_product(sigma_n, sigma_n);

    return quadric_from_coefficients_matrix(a, b, c);
}

/// 和平面类似，见上面的注释
/// p q r 都是位置坐标
fn triangle_quadric(p: vec3f, q: vec3f, r: vec3f) -> Quadric {
    let pxq = cross(p, q);
    let qxr = cross(q, r);
    let rxp = cross(r, p);

    let xsum = pxq + qxr + rxp;
    let det = dot(pxq, r);

    return quadric_from_coefficients_matrix(self_outer_product(xsum), xsum * det, det * det);
}

/// 和平面类似，见上面的注释
fn probabilistic_triangle_quadric(
    mean_p: vec3f,
    mean_q: vec3f,
    mean_r: vec3f,
    stddev: f32,
) -> Quadric {
    let sigma: f32 = stddev * stddev;

    let pxq = cross(mean_p, mean_q);
    let qxr = cross(mean_q, mean_r);
    let rxp = cross(mean_r, mean_p);

    let det_pqr = dot(pxq, mean_r);

    let cross_pqr = pxq + qxr + rxp;

    let pmq = mean_p - mean_q;
    let qmr = mean_q - mean_r;
    let rmp = mean_r - mean_p;

    var a = self_outer_product(cross_pqr) + (cross_product_squared_transpose(pmq) + cross_product_squared_transpose(qmr) + cross_product_squared_transpose(rmp)) * sigma;

    let ss = sigma * sigma;
    let ss6 = 6.0 * ss;
    let ss2 = 2.0 * ss;

    a[0].x += ss6;
    a[1].y += ss6;
    a[2].z += ss6;

    var b = cross_pqr * det_pqr;

    b -= (cross(pmq, pxq) + cross(qmr, qxr) + cross(rmp, rxp)) * sigma;

    b += (mean_p + mean_q + mean_r) * ss2;

    var c = det_pqr * det_pqr;
    c += (dot(pxq, pxq) + dot(qxr, qxr) + dot(rxp, rxp)) * sigma; // 3x (a x b)^T M_c (a x b)
    c += (dot(mean_p, mean_p) + dot(mean_q, mean_q) + dot(mean_r, mean_r)) * ss2; // 3x a^T Ci[S_b, S_c] a
    c += ss6 * sigma; // Tr[S_r Ci[S_p, S_q]]

    return quadric_from_coefficients_matrix(a, b, c);
}

    /// 和平面类似，见上面的注释
fn probabilistic_triangle_quadric_sigma(
    mean_p: vec3f,
    mean_q: vec3f,
    mean_r: vec3f,
    sigma_p: mat3x3f,
    sigma_q: mat3x3f,
    sigma_r: mat3x3f,
) -> Quadric {
    let pxq = cross(mean_p, mean_q);
    let qxr = cross(mean_q, mean_r);
    let rxp = cross(mean_r, mean_p);

    let det_pqr = dot(pxq, mean_r);

    let cross_pqr = pxq + qxr + rxp;

    let pmq = mean_p - mean_q;
    let qmr = mean_q - mean_r;
    let rmp = mean_r - mean_p;

    let ci_pq = cross_interference_matrix(sigma_p, sigma_q);
    let ci_qr = cross_interference_matrix(sigma_q, sigma_r);
    let ci_rp = cross_interference_matrix(sigma_r, sigma_p);

    var a = self_outer_product(cross_pqr);

    a -= first_order_tri_quad(pmq, sigma_r);
    a -= first_order_tri_quad(qmr, sigma_p);
    a -= first_order_tri_quad(rmp, sigma_q);

    a = a + ci_pq + ci_qr + ci_rp;

    a[1].x = a[0].y;
    a[2].x = a[0].z;
    a[2].y = a[1].z;

    var b = cross_pqr * det_pqr;

    b -= cross(pmq, sigma_r * pxq);
    b -= cross(qmr, sigma_p * qxr);
    b -= cross(rmp, sigma_q * rxp);

    b += ci_pq * mean_r;
    b += ci_qr * mean_p;
    b += ci_rp * mean_q;

    var c = det_pqr * det_pqr;

    c += dot(pxq, sigma_r * pxq);
    c += dot(qxr, sigma_p * qxr);
    c += dot(rxp, sigma_q * rxp);

    c += dot(mean_p, ci_qr * mean_p);
    c += dot(mean_q, ci_rp * mean_q);
    c += dot(mean_r, ci_pq * mean_r);

    c += trace_of_product(sigma_r, ci_pq);

    return quadric_from_coefficients_matrix(a, b, c);
}

/// 矩阵列主序
fn quadric_a(quadric: Quadric) -> mat3x3f {
    var a = mat3x3f();

    a[0][0] = quadric.a00f;
    a[0][1] = quadric.a01f;
    a[0][2] = quadric.a02f;
    a[1][0] = quadric.a01f;
    a[1][1] = quadric.a11f;
    a[1][2] = quadric.a12f;
    a[2][0] = quadric.a02f;
    a[2][1] = quadric.a12f;
    a[2][2] = quadric.a22f;

    return a;
}

fn quadric_b(quadric: Quadric) -> vec3<f32> {
    return vec3(quadric.b0f, quadric.b1f, quadric.b2f);
}

// Returns a point minimizing this quadric(预估点)
// Solving Ax = r with some common subexpressions precomputed
fn quadric_minimizer(quadric: Quadric) -> vec3f {
    let a = quadric.a00f;
    let b = quadric.a01f;
    let c = quadric.a02f;
    let d = quadric.a11f;
    let e = quadric.a12f;
    let f = quadric.a22f;
    let r0 = quadric.b0f;
    let r1 = quadric.b1f;
    let r2 = quadric.b2f;

    let ad = a * d;
    let ae = a * e;
    let af = a * f;

    let bc = b * c;
    let be = b * e;
    let bf = b * f;
    let df = d * f;
    let ce = c * e;
    let cd = c * d;

    let be_cd = be - cd;
    let bc_ae = bc - ae;
    let ce_bf = ce - bf;

        // 1.0 / determinant of A
    let denom = 1.0 / (a * df + 2.0 * b * ce - ae * e - bf * b - cd * c);
    let nom0 = r0 * (df - e * e) + r1 * ce_bf + r2 * be_cd;
    let nom1 = r0 * ce_bf + r1 * (af - c * c) + r2 * bc_ae;
    let nom2 = r0 * be_cd + r1 * bc_ae + r2 * (ad - b * b);

    return vec3(nom0 * denom, nom1 * denom, nom2 * denom);
}

    /// Residual L2 error as given by x^T A x - 2 r^T x + c
    /// 给定一个点，判断这个点的误差
fn quadric_residual_l2_error(quadric: Quadric, p: vec3f) -> f32 {
    let ax = vec3f(
        quadric.a00f * p.x + quadric.a01f * p.y + quadric.a02f * p.z,
        quadric.a01f * p.x + quadric.a11f * p.y + quadric.a12f * p.z,
        quadric.a02f * p.x + quadric.a12f * p.y + quadric.a22f * p.z,
    );

    return dot(p, ax) - 2.0 * (p.x * quadric.b0f + p.y * quadric.b1f + p.z * quadric.b2f) + quadric.c;
}

fn quadric_add_quadric(lhs: Quadric, rhs: Quadric) -> Quadric {
    return Quadric(
        lhs.a00f + rhs.a00f,
        lhs.a01f + rhs.a01f,
        lhs.a02f + rhs.a02f,
        lhs.a11f + rhs.a11f,
        lhs.a12f + rhs.a12f,
        lhs.a22f + rhs.a22f,
        lhs.b0f + rhs.b0f,
        lhs.b1f + rhs.b1f,
        lhs.b2f + rhs.b2f,
        lhs.c + rhs.c,
    );
}

fn quadric_sub_quadric(lhs: Quadric, rhs: Quadric) -> Quadric {
    return Quadric(
        lhs.a00f - rhs.a00f,
        lhs.a01f - rhs.a01f,
        lhs.a02f - rhs.a02f,
        lhs.a11f - rhs.a11f,
        lhs.a12f - rhs.a12f,
        lhs.a22f - rhs.a22f,
        lhs.b0f - rhs.b0f,
        lhs.b1f - rhs.b1f,
        lhs.b2f - rhs.b2f,
        lhs.c - rhs.c,
    );
}

fn quadric_neg(quadric: Quadric) -> Quadric {
    return Quadric(
        -quadric.a00f,
        -quadric.a01f,
        -quadric.a02f,
        -quadric.a11f,
        -quadric.a12f,
        -quadric.a22f,
        -quadric.b0f,
        -quadric.b1f,
        -quadric.b2f,
        -quadric.c,
    );
}

fn quadric_mul_f32(quadric: Quadric, rhs: f32) -> Quadric {
    return Quadric(
        quadric.a00f * rhs,
        quadric.a01f * rhs,
        quadric.a02f * rhs,
        quadric.a11f * rhs,
        quadric.a12f * rhs,
        quadric.a22f * rhs,
        quadric.b0f * rhs,
        quadric.b1f * rhs,
        quadric.b2f * rhs,
        quadric.c * rhs,
    );
}

fn quadric_div_f32(quadric: Quadric, divisor: f32) {
    return Quadric(
        quadric.a00f / divisor,
        quadric.a01f / divisor,
        quadric.a02f / divisor,
        quadric.a11f / divisor,
        quadric.a12f / divisor,
        quadric.a22f / divisor,
        quadric.b0f / divisor,
        quadric.b1f / divisor,
        quadric.b2f / divisor,
        quadric.c / divisor,
    );
}

fn quadric_mul_quadric(lhs: Quadric, rhs: Quadric) -> Quadric {
    return Quadric(
        lhs.a00f * rhs.a00f,
        lhs.a01f * rhs.a01f,
        lhs.a02f * rhs.a02f,
        lhs.a11f * rhs.a11f,
        lhs.a12f * rhs.a12f,
        lhs.a22f * rhs.a22f,
        lhs.b0f * rhs.b0f,
        lhs.b1f * rhs.b1f,
        lhs.b2f * rhs.b2f,
        lhs.c * rhs.c,
    );
}
