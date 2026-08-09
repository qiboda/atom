use crate::math;

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use bevy::{
    math::{Mat3A, Vec3A},
    reflect::Reflect,
};

pub(crate) type Pos3A = Vec3A;

// E[x] = x^T * A^T * A * x −2 * x^T * A^T * b + b^T * b
// and A = n, b = p * n, x是未知数
// Quadric中的a为A^T A, b为A^T b, c为b^T b
/// 二次误差函数（QEF）：`E(x) = xᵀAx − 2xᵀb + c`。
/// `A` 为对称 3×3 矩阵（仅存上三角 6 个系数），`b` 为 3 维向量，`c` 为标量。
/// 支持加法/减法/数乘等算术运算以合并多个 quadric，并通过 [`minimizer`](Self::minimizer) 求最小化点。
#[derive(Debug, Copy, Clone, PartialEq, Default, Reflect)]
pub struct Quadric {
    /// a is a symmetric 3x3 matrix
    pub(crate) a00: f32,
    pub(crate) a01: f32,
    pub(crate) a02: f32,
    pub(crate) a11: f32,
    pub(crate) a12: f32,
    pub(crate) a22: f32,

    pub(crate) b0: f32,
    pub(crate) b1: f32,
    pub(crate) b2: f32,

    pub(crate) c: f32,
}

impl Quadric {
    /// 由 10 个系数直接构造 Quadric：`A` 上三角 6 项（`a00, a01, a02, a11, a12, a22`）+ `b` 3 项 + `c`。
    #[allow(clippy::too_many_arguments)]
    pub fn from_coefficients(
        a00: f32,
        a01: f32,
        a02: f32,
        a11: f32,
        a12: f32,
        a22: f32,
        b0: f32,
        b1: f32,
        b2: f32,
        c: f32,
    ) -> Self {
        Self {
            a00,
            a01,
            a02,
            a11,
            a12,
            a22,
            b0,
            b1,
            b2,
            c,
        }
    }

    /// 由对称矩阵 `A`、向量 `b`、标量 `c` 构造 Quadric。
    pub fn from_coefficients_matrix(a: Mat3A, b: Vec3A, c: f32) -> Self {
        Self {
            a00: a.x_axis.x,
            a01: a.x_axis.y,
            a02: a.x_axis.z,
            a11: a.y_axis.y,
            a12: a.y_axis.z,
            a22: a.z_axis.z,
            b0: b[0],
            b1: b[1],
            b2: b[2],
            c,
        }
    }
}

impl Quadric {
    /// 生成一个点的quadric。
    pub fn point_quadric(p: Pos3A) -> Quadric {
        Quadric::from_coefficients_matrix(Mat3A::IDENTITY, p, 0.0)
    }

    /// 生成一个平面的quadric，这个平面的法向量是n，平面上的一个点是p
    /// 是一个经典版本的quadric，不考虑不确定性
    pub fn plane_quadric(p: Pos3A, n: Vec3A) -> Quadric {
        let d = p.dot(n);
        Quadric::from_coefficients_matrix(math::self_outer_product(n), n * d, d * d)
    }

    /// 生成一个平面的quadric，这个平面的法向量是mean_n，平面上的一个点是mean_p
    /// stddev_p和stddev_n分别表示mean_p和mean_n的标准差, stddev_p和stddev_n是各向同性的。
    pub fn probabilistic_plane_quadric(
        mean_p: Pos3A,
        mean_n: Vec3A,
        stddev_p: f32,
        stddev_n: f32,
    ) -> Quadric {
        let p = mean_p;
        let n = mean_n;

        let sn2 = stddev_n * stddev_n;
        let sp2 = stddev_p * stddev_p;

        let d = p.dot(n);

        let mut a = math::self_outer_product(n);

        a.x_axis.x += sn2;
        a.y_axis.y += sn2;
        a.z_axis.z += sn2;

        let b = n * d + p * sn2;
        let c = d * d + sn2 * p.dot(p) + sp2 * n.dot(n) + 3.0 * sp2 * sn2;

        Quadric::from_coefficients_matrix(a, b, c)
    }

    /// sigma => covariance matrix
    /// 生成一个平面的quadric，这个平面的法向量是mean_n，平面上的一个点是mean_p
    /// sigma_p和sigma_n分别表示mean_p和mean_n的协方差矩阵, sigma_p和sigma_n是各向异性的。
    pub fn probabilistic_plane_quadric_sigma(
        mean_p: Pos3A,
        mean_n: Vec3A,
        sigma_p: Mat3A,
        sigma_n: Mat3A,
    ) -> Quadric {
        let p = mean_p;
        let d = p.dot(mean_n);

        let a = math::self_outer_product(mean_n) + sigma_n;

        let b = mean_n * d + sigma_n * p;
        let c = d * d
            + p.dot(sigma_n * p)
            + mean_n.dot(sigma_p * mean_n)
            + math::trace_of_product(sigma_n, sigma_n);

        Quadric::from_coefficients_matrix(a, b, c)
    }

    /// 和平面类似，见上面的注释
    pub fn triangle_quadric(p: Pos3A, q: Pos3A, r: Pos3A) -> Quadric {
        let pxq = p.cross(q);
        let qxr = q.cross(r);
        let rxp = r.cross(p);

        let xsum = pxq + qxr + rxp;
        let det = pxq.dot(r);

        Quadric::from_coefficients_matrix(math::self_outer_product(xsum), xsum * det, det * det)
    }

    /// 和平面类似，见上面的注释
    pub fn probabilistic_triangle_quadric(
        mean_p: Pos3A,
        mean_q: Pos3A,
        mean_r: Pos3A,
        stddev: f32,
    ) -> Quadric {
        let sigma: f32 = stddev * stddev;

        let pxq = mean_p.cross(mean_q);
        let qxr = mean_q.cross(mean_r);
        let rxp = mean_r.cross(mean_p);

        let det_pqr = pxq.dot(mean_r);

        let cross_pqr = pxq + qxr + rxp;

        let pmq = mean_p - mean_q;
        let qmr = mean_q - mean_r;
        let rmp = mean_r - mean_p;

        let mut a = math::self_outer_product(cross_pqr)
            + (math::cross_product_squared_transpose(pmq)
                + math::cross_product_squared_transpose(qmr)
                + math::cross_product_squared_transpose(rmp))
                * sigma;

        let ss = sigma * sigma;
        let ss6 = 6.0 * ss;
        let ss2 = 2.0 * ss;

        a.x_axis.x += ss6;
        a.y_axis.y += ss6;
        a.z_axis.z += ss6;

        let mut b = cross_pqr * det_pqr;

        b -= (pmq.cross(pxq) + qmr.cross(qxr) + rmp.cross(rxp)) * sigma;

        b += (mean_p + mean_q + mean_r) * ss2;

        let mut c = det_pqr * det_pqr;
        c += (pxq.dot(pxq) + qxr.dot(qxr) + rxp.dot(rxp)) * sigma; // 3x (a x b)^T M_c (a x b)
        c += (mean_p.dot(mean_p) + mean_q.dot(mean_q) + mean_r.dot(mean_r)) * ss2; // 3x a^T Ci[S_b, S_c] a
        c += ss6 * sigma; // Tr[S_r Ci[S_p, S_q]]

        Quadric::from_coefficients_matrix(a, b, c)
    }

    /// 和平面类似，见上面的注释
    pub fn probabilistic_triangle_quadric_sigma(
        mean_p: Pos3A,
        mean_q: Pos3A,
        mean_r: Pos3A,
        sigma_p: Mat3A,
        sigma_q: Mat3A,
        sigma_r: Mat3A,
    ) -> Quadric {
        let pxq = mean_p.cross(mean_q);
        let qxr = mean_q.cross(mean_r);
        let rxp = mean_r.cross(mean_p);

        let det_pqr = pxq.dot(mean_r);

        let cross_pqr = pxq + qxr + rxp;

        let pmq = mean_p - mean_q;
        let qmr = mean_q - mean_r;
        let rmp = mean_r - mean_p;

        let ci_pq = math::cross_interference_matrix(sigma_p, sigma_q);
        let ci_qr = math::cross_interference_matrix(sigma_q, sigma_r);
        let ci_rp = math::cross_interference_matrix(sigma_r, sigma_p);

        let mut a = math::self_outer_product(cross_pqr);

        a -= math::first_order_tri_quad(pmq, sigma_r);
        a -= math::first_order_tri_quad(qmr, sigma_p);
        a -= math::first_order_tri_quad(rmp, sigma_q);

        a = a + ci_pq + ci_qr + ci_rp;

        a.y_axis.x = a.x_axis.y;
        a.z_axis.x = a.x_axis.z;
        a.z_axis.y = a.y_axis.z;

        let mut b = cross_pqr * det_pqr;

        b -= pmq.cross(sigma_r * pxq);
        b -= qmr.cross(sigma_p * qxr);
        b -= rmp.cross(sigma_q * rxp);

        b += ci_pq * mean_r;
        b += ci_qr * mean_p;
        b += ci_rp * mean_q;

        let mut c = det_pqr * det_pqr;

        c += pxq.dot(sigma_r * pxq);
        c += qxr.dot(sigma_p * qxr);
        c += rxp.dot(sigma_q * rxp);

        c += mean_p.dot(ci_qr * mean_p);
        c += mean_q.dot(ci_rp * mean_q);
        c += mean_r.dot(ci_pq * mean_r);

        c += math::trace_of_product(sigma_r, ci_pq);

        Quadric::from_coefficients_matrix(a, b, c)
    }
}

impl Quadric {
    /// 返回对称矩阵 `A`。
    pub fn a(&self) -> Mat3A {
        let mut a = Mat3A::default();

        a.x_axis.x = self.a00;
        a.x_axis.y = self.a01;
        a.x_axis.z = self.a02;
        a.y_axis.x = self.a01;
        a.y_axis.y = self.a11;
        a.y_axis.z = self.a12;
        a.z_axis.x = self.a02;
        a.z_axis.y = self.a12;
        a.z_axis.z = self.a22;

        a
    }

    /// 返回向量 `b`。
    pub fn b(&self) -> Vec3A {
        Vec3A::new(self.b0, self.b1, self.b2)
    }

    // Returns a point minimizing this quadric(预估点)
    // Solving Ax = r with some common subexpressions precomputed
    /// 返回最小化该 quadric 的点（求解 `Ax = b`）。
    /// 注意：`A` 奇异（如仅有一个平面的约束）时结果无定义。
    pub fn minimizer(&self) -> Vec3A {
        let a = self.a00;
        let b = self.a01;
        let c = self.a02;
        let d = self.a11;
        let e = self.a12;
        let f = self.a22;
        let r0 = self.b0;
        let r1 = self.b1;
        let r2 = self.b2;

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

        Vec3A::new(nom0 * denom, nom1 * denom, nom2 * denom)
    }

    /// Residual L2 error as given by x^T A x - 2 r^T x + c
    /// 给定一个点，判断这个点的误差
    pub fn residual_l2_error(&self, p: Pos3A) -> f32 {
        let ax = Vec3A::new(
            self.a00 * p.x + self.a01 * p.y + self.a02 * p.z,
            self.a01 * p.x + self.a11 * p.y + self.a12 * p.z,
            self.a02 * p.x + self.a12 * p.y + self.a22 * p.z,
        );

        p.dot(ax) - 2.0 * (p.x * self.b0 + p.y * self.b1 + p.z * self.b2) + self.c
    }
}

impl AddAssign for Quadric {
    fn add_assign(&mut self, rhs: Self) {
        self.a00 += rhs.a00;
        self.a01 += rhs.a01;
        self.a02 += rhs.a02;
        self.a11 += rhs.a11;
        self.a12 += rhs.a12;
        self.a22 += rhs.a22;
        self.b0 += rhs.b0;
        self.b1 += rhs.b1;
        self.b2 += rhs.b2;
        self.c += rhs.c;
    }
}

impl SubAssign for Quadric {
    fn sub_assign(&mut self, rhs: Self) {
        self.a00 -= rhs.a00;
        self.a01 -= rhs.a01;
        self.a02 -= rhs.a02;
        self.a11 -= rhs.a11;
        self.a12 -= rhs.a12;
        self.a22 -= rhs.a22;
        self.b0 -= rhs.b0;
        self.b1 -= rhs.b1;
        self.b2 -= rhs.b2;
        self.c -= rhs.c;
    }
}

impl MulAssign<f32> for Quadric {
    fn mul_assign(&mut self, rhs: f32) {
        self.a00 *= rhs;
        self.a01 *= rhs;
        self.a02 *= rhs;
        self.a11 *= rhs;
        self.a12 *= rhs;
        self.a22 *= rhs;
        self.b0 *= rhs;
        self.b1 *= rhs;
        self.b2 *= rhs;
        self.c *= rhs;
    }
}

impl DivAssign<f32> for Quadric {
    fn div_assign(&mut self, rhs: f32) {
        self.a00 /= rhs;
        self.a01 /= rhs;
        self.a02 /= rhs;
        self.a11 /= rhs;
        self.a12 /= rhs;
        self.a22 /= rhs;
        self.b0 /= rhs;
        self.b1 /= rhs;
        self.b2 /= rhs;
        self.c /= rhs;
    }
}

impl Add for Quadric {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            a00: self.a00 + rhs.a00,
            a01: self.a01 + rhs.a01,
            a02: self.a02 + rhs.a02,
            a11: self.a11 + rhs.a11,
            a12: self.a12 + rhs.a12,
            a22: self.a22 + rhs.a22,
            b0: self.b0 + rhs.b0,
            b1: self.b1 + rhs.b1,
            b2: self.b2 + rhs.b2,
            c: self.c + rhs.c,
        }
    }
}

impl Sub for Quadric {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            a00: self.a00 - rhs.a00,
            a01: self.a01 - rhs.a01,
            a02: self.a02 - rhs.a02,
            a11: self.a11 - rhs.a11,
            a12: self.a12 - rhs.a12,
            a22: self.a22 - rhs.a22,
            b0: self.b0 - rhs.b0,
            b1: self.b1 - rhs.b1,
            b2: self.b2 - rhs.b2,
            c: self.c - rhs.c,
        }
    }
}

impl Neg for Quadric {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            a00: -self.a00,
            a01: -self.a01,
            a02: -self.a02,
            a11: -self.a11,
            a12: -self.a12,
            a22: -self.a22,
            b0: -self.b0,
            b1: -self.b1,
            b2: -self.b2,
            c: -self.c,
        }
    }
}

impl Mul<f32> for Quadric {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            a00: self.a00 * rhs,
            a01: self.a01 * rhs,
            a02: self.a02 * rhs,
            a11: self.a11 * rhs,
            a12: self.a12 * rhs,
            a22: self.a22 * rhs,
            b0: self.b0 * rhs,
            b1: self.b1 * rhs,
            b2: self.b2 * rhs,
            c: self.c * rhs,
        }
    }
}

impl Div<f32> for Quadric {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            a00: self.a00 / rhs,
            a01: self.a01 / rhs,
            a02: self.a02 / rhs,
            a11: self.a11 / rhs,
            a12: self.a12 / rhs,
            a22: self.a22 / rhs,
            b0: self.b0 / rhs,
            b1: self.b1 / rhs,
            b2: self.b2 / rhs,
            c: self.c / rhs,
        }
    }
}

impl Mul<Quadric> for f32 {
    type Output = Quadric;

    fn mul(self, rhs: Quadric) -> Self::Output {
        Quadric {
            a00: self * rhs.a00,
            a01: self * rhs.a01,
            a02: self * rhs.a02,
            a11: self * rhs.a11,
            a12: self * rhs.a12,
            a22: self * rhs.a22,
            b0: self * rhs.b0,
            b1: self * rhs.b1,
            b2: self * rhs.b2,
            c: self * rhs.c,
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use bevy::math::{Mat3A, Vec3A};

    use crate::{math::covariance_matrix, quadric::Quadric};

    #[test]
    fn test_point_quadric() {
        let p = Vec3A::new(1.0, 2.0, 3.0);
        let q = Quadric::point_quadric(p);
        assert_eq!(q.a(), Mat3A::IDENTITY);
        assert_eq!(q.b(), p);
        assert_eq!(q.c, 0.0);
    }

    #[test]
    fn test_plane_quadric_minimizer() {
        // Combine three orthogonal plane quadrics to get a full-rank A matrix
        // Intersection of x=1, y=5, z=0 should give minimizer near (1, 5, 0)
        let q1 = Quadric::plane_quadric(Vec3A::new(0.0, 5.0, 0.0), Vec3A::new(0.0, 1.0, 0.0));
        let q2 = Quadric::plane_quadric(Vec3A::new(1.0, 0.0, 0.0), Vec3A::new(1.0, 0.0, 0.0));
        let q3 = Quadric::plane_quadric(Vec3A::new(0.0, 0.0, 0.0), Vec3A::new(0.0, 0.0, 1.0));
        let q = q1 + q2 + q3;
        let min = q.minimizer();
        assert!(
            (min.y - 5.0).abs() < 1e-4,
            "minimizer y={}, expected 5.0",
            min.y
        );
        assert!(
            (min.x - 1.0).abs() < 1e-4,
            "minimizer x={}, expected 1.0",
            min.x
        );
    }

    #[test]
    fn test_plane_quadric_residual_at_minimizer() {
        // Use three orthogonal planes through origin for full-rank system
        let q1 = Quadric::plane_quadric(Vec3A::new(0.0, 0.0, 0.0), Vec3A::new(1.0, 0.0, 0.0));
        let q2 = Quadric::plane_quadric(Vec3A::new(0.0, 0.0, 0.0), Vec3A::new(0.0, 1.0, 0.0));
        let q3 = Quadric::plane_quadric(Vec3A::new(0.0, 0.0, 0.0), Vec3A::new(0.0, 0.0, 1.0));
        let q = q1 + q2 + q3;
        let min = q.minimizer();
        let err = q.residual_l2_error(min);
        assert!(
            err.abs() < 1e-6,
            "residual at minimizer = {}, expected ~0",
            err
        );
    }

    #[test]
    fn test_single_quadric() {
        let quadric = Quadric::probabilistic_plane_quadric(
            Vec3A::new(0.0, 0.0, 0.0),
            Vec3A::new(0.0, 0.0, 1.0),
            0.1,
            0.1,
        );
        let pos = quadric.minimizer();
        let error = quadric.residual_l2_error(pos);
        // minimizer should be near origin for a plane through origin
        assert!(pos.length() < 0.5, "minimizer too far from origin: {}", pos);
        assert!(error >= 0.0, "residual should be non-negative");
    }

    #[test]
    fn test_two_quadric_combination() {
        let q1 = Quadric::probabilistic_plane_quadric(
            Vec3A::new(1.0, 0.0, 0.0),
            Vec3A::new(1.0, 0.0, 0.0),
            0.1,
            0.0,
        );
        let q2 = Quadric::probabilistic_plane_quadric(
            Vec3A::new(0.0, 1.0, 0.0),
            Vec3A::new(0.0, 1.0, 0.0),
            0.0,
            0.1,
        );
        let combined = q1 + q2;
        let pos = combined.minimizer();
        // Should be near (1, 1, 0) - intersection of x=1 and y=1 planes
        assert!((pos.x - 1.0).abs() < 0.5, "x={}, expected ~1.0", pos.x);
        assert!((pos.y - 1.0).abs() < 0.5, "y={}, expected ~1.0", pos.y);
    }

    #[test]
    fn test_quadric_arithmetic() {
        let q = Quadric::point_quadric(Vec3A::new(1.0, 2.0, 3.0));
        let neg_q = -q;
        let zero = q + neg_q;
        assert!(zero.a00.abs() < 1e-6);
        assert!(zero.b0.abs() < 1e-6);
        assert!(zero.c.abs() < 1e-6);
    }

    #[test]
    fn test_quadric_scalar_mul_div() {
        let q = Quadric::point_quadric(Vec3A::new(1.0, 0.0, 0.0));
        let q2 = q * 2.0;
        let q_back = q2 / 2.0;
        assert!((q_back.a00 - q.a00).abs() < 1e-6);
        assert!((q_back.b0 - q.b0).abs() < 1e-6);
    }

    #[test]
    fn test_quadric_a_symmetry() {
        let q = Quadric::probabilistic_plane_quadric(
            Vec3A::new(1.0, 2.0, 3.0),
            Vec3A::new(0.5, 0.5, 0.707).normalize(),
            0.1,
            0.1,
        );
        let a = q.a();
        let cols = a.to_cols_array_2d();
        assert!((cols[0][1] - cols[1][0]).abs() < 1e-6);
        assert!((cols[0][2] - cols[2][0]).abs() < 1e-6);
        assert!((cols[1][2] - cols[2][1]).abs() < 1e-6);
    }

    #[test]
    fn test_probabilistic_plane_quadric_sigma() {
        let positions = [
            Vec3A::new(0.0, 0.0, 1.0),
            Vec3A::new(1.0, 0.0, 0.0),
            Vec3A::new(0.0, 1.0, 0.3),
            Vec3A::new(0.5, 0.5, 0.0),
        ];
        let pos_mat = covariance_matrix(&positions);
        let mean_pos = positions.iter().fold(Vec3A::ZERO, |acc, &x| acc + x) / 4.0;

        let normals = [
            Vec3A::new(1.0, 1.0, 0.2).normalize(),
            Vec3A::new(0.0, 1.0, 0.0).normalize(),
            Vec3A::new(1.0, 0.0, 0.5).normalize(),
            Vec3A::new(0.25, 0.25, 0.0).normalize(),
        ];
        let normal_mat = covariance_matrix(&normals);
        let mean_normal: Vec3A =
            (normals.iter().fold(Vec3A::ZERO, |acc, &x| acc + x) / 4.0).normalize();

        let quadric =
            Quadric::probabilistic_plane_quadric_sigma(mean_pos, mean_normal, pos_mat, normal_mat);
        let pos = quadric.minimizer();
        let error = quadric.residual_l2_error(pos);
        // Minimizer should exist and error should be non-negative
        assert!(pos.is_finite(), "minimizer should be finite");
        assert!(
            error >= -1e-4,
            "residual should be approximately non-negative"
        );
    }

    #[test]
    fn test_from_coefficients_roundtrip() {
        let q = Quadric::from_coefficients(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0);
        let a = q.a().to_cols_array_2d();
        assert!((a[0][0] - 1.0).abs() < 1e-6);
        assert!((a[0][1] - 2.0).abs() < 1e-6);
        assert!((a[0][2] - 3.0).abs() < 1e-6);
        assert!((a[1][0] - 2.0).abs() < 1e-6);
        assert!((a[1][1] - 4.0).abs() < 1e-6);
        assert!((a[1][2] - 5.0).abs() < 1e-6);
        assert!((a[2][0] - 3.0).abs() < 1e-6);
        assert!((a[2][1] - 5.0).abs() < 1e-6);
        assert!((a[2][2] - 6.0).abs() < 1e-6);
        assert_eq!(q.b(), Vec3A::new(7.0, 8.0, 9.0));
        assert!((q.c - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_from_coefficients_matrix_reads_upper_triangle() {
        // 非对称输入矩阵：只读取上三角（含主对角线），下三角被丢弃。
        let m = Mat3A::from_cols(
            Vec3A::new(1.0, 9.0, 8.0),
            Vec3A::new(2.0, 4.0, 7.0),
            Vec3A::new(3.0, 5.0, 6.0),
        );
        let q = Quadric::from_coefficients_matrix(m, Vec3A::new(1.0, 2.0, 3.0), 4.0);
        let a = q.a().to_cols_array_2d();
        // a01 = 9 / a02 = 8 / a12 = 7（来自输入上三角），而不是下三角的 2/3/5。
        assert!((a[0][0] - 1.0).abs() < 1e-6);
        assert!((a[0][1] - 9.0).abs() < 1e-6);
        assert!((a[0][2] - 8.0).abs() < 1e-6);
        assert!((a[1][0] - 9.0).abs() < 1e-6);
        assert!((a[1][1] - 4.0).abs() < 1e-6);
        assert!((a[1][2] - 7.0).abs() < 1e-6);
        assert!((a[2][0] - 8.0).abs() < 1e-6);
        assert!((a[2][1] - 7.0).abs() < 1e-6);
        assert!((a[2][2] - 6.0).abs() < 1e-6);
        assert_eq!(q.b(), Vec3A::new(1.0, 2.0, 3.0));
        assert!((q.c - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_default_quadric_is_zero() {
        let q = Quadric::default();
        assert_eq!(q.a(), Mat3A::ZERO);
        assert_eq!(q.b(), Vec3A::ZERO);
        assert_eq!(q.c, 0.0);
        assert_eq!(
            q,
            Quadric::from_coefficients(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
        );
    }

    #[test]
    fn test_triangle_quadric_coefficients() {
        // p=(1,0,0), q=(0,1,0), r=(0,0,1)：
        // cross_pqr = (1,1,1)，det = 1 → a = outer((1,1,1))，b = (1,1,1)，c = 1。
        let q = Quadric::triangle_quadric(
            Vec3A::new(1.0, 0.0, 0.0),
            Vec3A::new(0.0, 1.0, 0.0),
            Vec3A::new(0.0, 0.0, 1.0),
        );
        let a = q.a().to_cols_array_2d();
        for row in a {
            for v in row {
                assert!((v - 1.0).abs() < 1e-6, "a value = {v}");
            }
        }
        assert_eq!(q.b(), Vec3A::splat(1.0));
        assert!((q.c - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_triangle_quadric_residual_on_plane() {
        // 平面 x+y+z=1 上的点（含三个顶点）残差应为 0。
        let q = Quadric::triangle_quadric(
            Vec3A::new(1.0, 0.0, 0.0),
            Vec3A::new(0.0, 1.0, 0.0),
            Vec3A::new(0.0, 0.0, 1.0),
        );
        for p in [
            Vec3A::new(1.0, 0.0, 0.0),
            Vec3A::new(0.0, 1.0, 0.0),
            Vec3A::new(0.0, 0.0, 1.0),
            Vec3A::new(0.25, 0.25, 0.5),
            Vec3A::splat(1.0 / 3.0),
        ] {
            assert!(
                q.residual_l2_error(p).abs() < 1e-6,
                "residual on plane should be 0, got {}",
                q.residual_l2_error(p)
            );
        }
        // 平面外任意点：E = (n·x − d)² ≥ 0。
        assert!(q.residual_l2_error(Vec3A::new(2.0, 0.0, 0.0)) > 0.0);
        assert!(q.residual_l2_error(Vec3A::new(-1.0, -1.0, -1.0)) > 0.0);
    }

    #[test]
    fn test_probabilistic_triangle_quadric_zero_stddev_matches_triangle() {
        let p = Vec3A::new(1.0, 0.0, 0.0);
        let q = Vec3A::new(0.0, 1.0, 0.0);
        let r = Vec3A::new(0.0, 0.0, 1.0);
        assert_eq!(
            Quadric::triangle_quadric(p, q, r),
            Quadric::probabilistic_triangle_quadric(p, q, r, 0.0)
        );
    }

    #[test]
    fn test_probabilistic_triangle_quadric_zero_sigma_matches_triangle() {
        let p = Vec3A::new(1.0, 0.0, 0.0);
        let q = Vec3A::new(0.0, 1.0, 0.0);
        let r = Vec3A::new(0.0, 0.0, 1.0);
        assert_eq!(
            Quadric::triangle_quadric(p, q, r),
            Quadric::probabilistic_triangle_quadric_sigma(
                p,
                q,
                r,
                Mat3A::ZERO,
                Mat3A::ZERO,
                Mat3A::ZERO,
            )
        );
    }

    #[test]
    fn test_probabilistic_triangle_quadric_symmetry_and_residual() {
        let q = Quadric::probabilistic_triangle_quadric(
            Vec3A::new(1.0, 0.0, 0.0),
            Vec3A::new(0.0, 1.0, 0.0),
            Vec3A::new(0.0, 0.0, 1.0),
            0.1,
        );
        let cols = q.a().to_cols_array_2d();
        assert!((cols[0][1] - cols[1][0]).abs() < 1e-6);
        assert!((cols[0][2] - cols[2][0]).abs() < 1e-6);
        assert!((cols[1][2] - cols[2][1]).abs() < 1e-6);
        // 概率化是平方误差的期望，残差应非负。
        for p in [
            Vec3A::new(1.0, 0.0, 0.0),
            Vec3A::new(0.0, 1.0, 0.0),
            Vec3A::new(0.0, 0.0, 1.0),
            Vec3A::new(0.4, 0.4, 0.4),
        ] {
            let e = q.residual_l2_error(p);
            assert!(e >= -1e-3, "residual {e} < -1e-3");
        }
    }

    #[test]
    fn test_probabilistic_triangle_quadric_sigma_symmetry_and_residual() {
        let q = Quadric::probabilistic_triangle_quadric_sigma(
            Vec3A::new(1.0, 0.0, 0.0),
            Vec3A::new(0.0, 1.0, 0.0),
            Vec3A::new(0.0, 0.0, 1.0),
            Mat3A::from_cols(Vec3A::splat(0.01), Vec3A::splat(0.01), Vec3A::splat(0.01)),
            Mat3A::from_cols(Vec3A::splat(0.01), Vec3A::splat(0.01), Vec3A::splat(0.01)),
            Mat3A::from_cols(Vec3A::splat(0.01), Vec3A::splat(0.01), Vec3A::splat(0.01)),
        );
        let cols = q.a().to_cols_array_2d();
        assert!((cols[0][1] - cols[1][0]).abs() < 1e-6);
        assert!((cols[0][2] - cols[2][0]).abs() < 1e-6);
        assert!((cols[1][2] - cols[2][1]).abs() < 1e-6);
        let e = q.residual_l2_error(Vec3A::new(1.0, 0.0, 0.0));
        assert!(e >= -1e-3, "residual {e} < -1e-3");
    }

    #[test]
    fn test_minimizer_identity_matrix() {
        let q = Quadric::from_coefficients_matrix(Mat3A::IDENTITY, Vec3A::new(1.0, 2.0, 3.0), 5.0);
        let min = q.minimizer();
        assert!((min.x - 1.0).abs() < 1e-5, "x={}", min.x);
        assert!((min.y - 2.0).abs() < 1e-5, "y={}", min.y);
        assert!((min.z - 3.0).abs() < 1e-5, "z={}", min.z);
    }

    #[test]
    fn test_minimizer_diagonal_matrix() {
        // A = diag(2,4,6)，b = (2,4,6) → x = (1,1,1)。
        let q = Quadric::from_coefficients(2.0, 0.0, 0.0, 4.0, 0.0, 6.0, 2.0, 4.0, 6.0, 0.0);
        let min = q.minimizer();
        assert!((min.x - 1.0).abs() < 1e-5, "x={}", min.x);
        assert!((min.y - 1.0).abs() < 1e-5, "y={}", min.y);
        assert!((min.z - 1.0).abs() < 1e-5, "z={}", min.z);
    }

    #[test]
    fn test_minimizer_point_quadric_returns_point() {
        let p = Vec3A::new(-2.0, 3.5, 0.25);
        let min = Quadric::point_quadric(p).minimizer();
        assert!((min - p).length() < 1e-4, "minimizer={min}, expected ~{p}");
    }

    #[test]
    fn test_minimizer_singular_matrix_does_not_panic() {
        // 单个平面约束 → A 奇异，denom = 1/0 = inf → 结果非有限（但不应 panic）。
        let q = Quadric::plane_quadric(Vec3A::new(0.0, 5.0, 0.0), Vec3A::new(0.0, 1.0, 0.0));
        let min = q.minimizer();
        assert!(
            !min.is_finite(),
            "singular minimizer should not be finite: {min}"
        );

        // 全零 quadric：0 * inf = NaN。
        assert!(!Quadric::default().minimizer().is_finite());
    }

    #[test]
    fn test_residual_matches_manual_formula() {
        let q = Quadric::from_coefficients(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0);
        let p = Vec3A::new(0.5, -1.0, 2.0);
        // E = xᵀAx − 2xᵀb + c（A 对称）。
        let ax = q.a() * p;
        let manual = p.dot(ax) - 2.0 * p.dot(q.b()) + q.c;
        assert!((q.residual_l2_error(p) - manual).abs() < 1e-4);
    }

    #[test]
    fn test_point_quadric_residual() {
        let p = Vec3A::new(1.0, 2.0, 3.0);
        let q = Quadric::point_quadric(p);
        // E(x) = ||x||² − 2x·p：在 x=p 处 = −||p||²，在 x=0 处 = 0。
        assert!((q.residual_l2_error(p) + p.length_squared()).abs() < 1e-4);
        assert!(q.residual_l2_error(Vec3A::ZERO).abs() < 1e-4);
    }

    #[test]
    fn test_plane_quadric_residual_vanishes_on_plane() {
        let q = Quadric::plane_quadric(Vec3A::new(0.0, 5.0, 0.0), Vec3A::new(0.0, 1.0, 0.0));
        // 平面上任意点（y=5）残差为 0；平面外 E = (n·x − d)² ≥ 0。
        for p in [
            Vec3A::new(0.0, 5.0, 0.0),
            Vec3A::new(3.0, 5.0, -2.0),
            Vec3A::new(-1.0, 5.0, 7.0),
        ] {
            assert!(q.residual_l2_error(p).abs() < 1e-5);
        }
        assert!(q.residual_l2_error(Vec3A::new(0.0, 6.0, 0.0)) > 0.0);
    }

    #[test]
    fn test_add_assign_matches_add() {
        let q1 = Quadric::plane_quadric(Vec3A::new(1.0, 0.0, 0.0), Vec3A::new(1.0, 0.0, 0.0));
        let q2 = Quadric::plane_quadric(Vec3A::new(0.0, 1.0, 0.0), Vec3A::new(0.0, 1.0, 0.0));
        let mut acc = q1;
        acc += q2;
        assert_eq!(acc, q1 + q2);
    }

    #[test]
    fn test_sub_assign_matches_sub() {
        let q1 = Quadric::plane_quadric(Vec3A::new(1.0, 0.0, 0.0), Vec3A::new(1.0, 0.0, 0.0));
        let q2 = Quadric::plane_quadric(Vec3A::new(0.0, 1.0, 0.0), Vec3A::new(0.0, 1.0, 0.0));
        let mut acc = q1;
        acc -= q2;
        assert_eq!(acc, q1 - q2);
    }

    #[test]
    fn test_mul_assign_matches_mul() {
        let q = Quadric::plane_quadric(Vec3A::new(1.0, 0.0, 0.0), Vec3A::new(1.0, 0.0, 0.0));
        let mut acc = q;
        acc *= 2.5;
        assert_eq!(acc, q * 2.5);
    }

    #[test]
    fn test_div_assign_matches_div() {
        let q = Quadric::plane_quadric(Vec3A::new(1.0, 0.0, 0.0), Vec3A::new(1.0, 0.0, 0.0));
        let mut acc = q;
        acc /= 2.5;
        assert_eq!(acc, q / 2.5);
    }

    #[test]
    fn test_scalar_left_mul_commutes() {
        let q = Quadric::plane_quadric(Vec3A::new(1.0, 0.0, 0.0), Vec3A::new(1.0, 0.0, 0.0));
        assert_eq!(2.5 * q, q * 2.5);
        assert_eq!((-3.0) * q, q * (-3.0));
    }

    #[test]
    fn test_neg_flips_all_coefficients() {
        let q = Quadric::from_coefficients(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0);
        let neg = -q;
        let a = neg.a().to_cols_array_2d();
        assert!((a[0][0] + 1.0).abs() < 1e-6);
        assert!((a[0][1] + 2.0).abs() < 1e-6);
        assert!((a[2][2] + 6.0).abs() < 1e-6);
        assert_eq!(neg.b(), Vec3A::new(-7.0, -8.0, -9.0));
        assert!((neg.c + 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_residual_add_linearity() {
        // (a + b).eval(p) == a.eval(p) + b.eval(p)。
        let a = Quadric::plane_quadric(Vec3A::new(1.0, 0.0, 0.0), Vec3A::new(1.0, 0.0, 0.0));
        let b = Quadric::probabilistic_plane_quadric(
            Vec3A::new(0.0, 1.0, 0.0),
            Vec3A::new(0.0, 1.0, 0.0),
            0.1,
            0.1,
        );
        for p in [
            Vec3A::new(0.3, -0.7, 1.2),
            Vec3A::new(-2.0, 4.0, 0.5),
            Vec3A::splat(0.1),
        ] {
            let lhs = (a + b).residual_l2_error(p);
            let rhs = a.residual_l2_error(p) + b.residual_l2_error(p);
            assert!(
                (lhs - rhs).abs() < 1e-4,
                "add linearity lhs={lhs} rhs={rhs}"
            );
        }
    }

    #[test]
    fn test_residual_sub_linearity() {
        let a = Quadric::plane_quadric(Vec3A::new(1.0, 0.0, 0.0), Vec3A::new(1.0, 0.0, 0.0));
        let b = Quadric::plane_quadric(Vec3A::new(0.0, 1.0, 0.0), Vec3A::new(0.0, 1.0, 0.0));
        let p = Vec3A::new(0.3, -0.7, 1.2);
        let lhs = (a - b).residual_l2_error(p);
        let rhs = a.residual_l2_error(p) - b.residual_l2_error(p);
        assert!(
            (lhs - rhs).abs() < 1e-4,
            "sub linearity lhs={lhs} rhs={rhs}"
        );
    }

    #[test]
    fn test_residual_scale_linearity() {
        let a = Quadric::plane_quadric(Vec3A::new(1.0, 0.0, 0.0), Vec3A::new(1.0, 0.0, 0.0));
        let p = Vec3A::new(0.3, -0.7, 1.2);
        let s = 2.5;
        assert!(((a * s).residual_l2_error(p) - s * a.residual_l2_error(p)).abs() < 1e-4);
        assert!(((a / s).residual_l2_error(p) - a.residual_l2_error(p) / s).abs() < 1e-4);
        assert!(((-a).residual_l2_error(p) + a.residual_l2_error(p)).abs() < 1e-4);
    }

    #[test]
    fn test_merge_and_normalize_quadrics() {
        // 合并三个正交平面 quadric 并除以数量（规范化平均），minimizer 应保持在交点。
        let planes = [
            Quadric::plane_quadric(Vec3A::new(1.0, 0.0, 0.0), Vec3A::new(1.0, 0.0, 0.0)),
            Quadric::plane_quadric(Vec3A::new(0.0, 1.0, 0.0), Vec3A::new(0.0, 1.0, 0.0)),
            Quadric::plane_quadric(Vec3A::new(0.0, 0.0, 1.0), Vec3A::new(0.0, 0.0, 1.0)),
        ];
        let mut merged = Quadric::default();
        for q in planes {
            merged += q;
        }
        let normalized = merged / 3.0;
        let m = normalized.minimizer();
        assert!((m.x - 1.0).abs() < 1e-4, "x={}", m.x);
        assert!((m.y - 1.0).abs() < 1e-4, "y={}", m.y);
        assert!((m.z - 1.0).abs() < 1e-4, "z={}", m.z);
    }
}
