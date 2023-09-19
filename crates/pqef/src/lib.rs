#![allow(dead_code)]

pub mod math;

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use bevy_math::{Mat3A, Vec3A};

type Pos3A = Vec3A;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Quadric {
    /// a is a symmetric 3x3 matrix
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
}

impl Quadric {
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
    pub fn point_quadric(p: Pos3A) -> Quadric {
        Quadric::from_coefficients_matrix(Mat3A::IDENTITY, p, 0.0)
    }

    pub fn plane_quadric(p: Pos3A, n: Vec3A) -> Quadric {
        let d = p.dot(n);
        Quadric::from_coefficients_matrix(math::self_outer_product(n), n * d, d * d)
    }

    /// stddev => standard deviation
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

    pub fn triangle_quadric(p: Pos3A, q: Pos3A, r: Pos3A) -> Quadric {
        let pxq = p.cross(q);
        let qxr = q.cross(r);
        let rxp = r.cross(p);

        let xsum = pxq + qxr + rxp;
        let det = pxq.dot(r);

        Quadric::from_coefficients_matrix(math::self_outer_product(xsum), xsum * det, det * det)
    }

    pub fn probabilistic_triangle_quadric(
        mean_p: Pos3A,
        mean_q: Pos3A,
        mean_r: Pos3A,
        stddev: f32,
    ) -> Quadric {
        let sigma = stddev * stddev;

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

        let c = det_pqr * det_pqr
            + (pxq.dot(pxq) + qxr.dot(qxr) + rxp.dot(rxp)) * sigma
            + (mean_p.dot(mean_p) + mean_q.dot(mean_q) + mean_r.dot(mean_r)) * ss2
            + ss6 * sigma;

        Quadric::from_coefficients_matrix(a, b, c)
    }

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

    pub fn b(&self) -> Vec3A {
        Vec3A::new(self.b0, self.b1, self.b2)
    }

    /// A^-1 * b
    /// A^-1 = A* / det(A)
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
mod tests {

    #[test]
    fn construct() {
        // Quadric::<f32>::from_coefficients(
        //     1.0, 2.0, 3.0, 4.0, 5.0, 6.0, //
        //     7.0, 8.0, 9.0, 10.0,
        // );
    }
}
