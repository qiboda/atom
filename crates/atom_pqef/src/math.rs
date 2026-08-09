use bevy::math::{Mat3A, Vec3A};

/// two vectors cross product, 3x1 * 1x3
pub(crate) fn self_outer_product(v: Vec3A) -> Mat3A {
    let a = v.x;
    let b = v.y;
    let c = v.z;

    Mat3A::from_cols(
        Vec3A::new(a * a, a * b, a * c),
        Vec3A::new(b * a, b * b, b * c),
        Vec3A::new(c * a, c * b, c * c),
    )
}

/// trace is Tr(A) = sigma(i=0..n) A_ii
/// Tr(AB) = Tr(BA) = Tr(BA^T) = Tr(A^TB)
/// Tr(AB) = sigma(i=0..n) sigma(j=0..m) A_ij B_ji
pub(crate) fn trace_of_product(a: Mat3A, b: Mat3A) -> f32 {
    let mut r = 0.0;
    for i in 0..3 {
        for j in 0..3 {
            r += a.to_cols_array_2d()[i][j] * b.to_cols_array_2d()[j][i];
        }
    }
    r
}

/// TODO: to understand
/// p and q are 3x3 matrices
/// and use p and q 2x2 matrices interference determinant to as interference value at non 2x2
/// matrix row and column
/// such as : m00 = (p11 * q22 - p12 * q21) + (p21 * q12 - p22 * q11)
pub(crate) fn cross_interference_matrix(p: Mat3A, q: Mat3A) -> Mat3A {
    let mut m = Mat3A::default();

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

pub(crate) fn first_order_tri_quad(a: Vec3A, sigma: Mat3A) -> Mat3A {
    let mut m = Mat3A::default();

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

/// v => [v]x => [v]x * [v]x^T
/// [v]x is skwy-symmetric matrix
pub(crate) fn cross_product_squared_transpose(v: Vec3A) -> Mat3A {
    let a = v[0];
    let b = v[1];
    let c = v[2];
    let a2 = a * a;
    let b2 = b * b;
    let c2 = c * c;

    let mut m: Mat3A = Default::default();

    m.x_axis.x = b2 + c2;
    m.y_axis.y = a2 + c2;
    m.z_axis.z = a2 + b2;

    m.y_axis.x = -a * b;
    m.z_axis.x = -a * c;
    m.z_axis.y = -b * c;

    m.x_axis.y = -a * b;
    m.x_axis.z = -a * c;
    m.y_axis.z = -b * c;

    m
}

/// 标准差：总体方差（`variance`）的平方根。
pub fn standard_deviation(sampler: &[f32]) -> f32 {
    variance(sampler).sqrt()
}

/// 总体方差：E(X²) − E(X)²。
pub fn variance(sampler: &[f32]) -> f32 {
    let len = sampler.iter().len();
    let mean = sampler.iter().sum::<f32>() / len as f32;

    let mut sum = 0.0;
    for sample in sampler.iter() {
        sum += sample * sample;
    }
    sum / len as f32 - mean * mean
}

/// 两个样本序列 `xs`/`ys` 的协方差（要求等长）。
pub fn covariance(xs: &[f32], ys: &[f32]) -> f32 {
    let x_len = xs.iter().len();
    let x_mean = xs.iter().sum::<f32>() / x_len as f32;

    let y_len = ys.iter().len();
    let y_mean = ys.iter().sum::<f32>() / y_len as f32;

    let xy_mean = xs.iter().zip(ys.iter()).map(|(x, y)| x * y).sum::<f32>() / x_len as f32;

    xy_mean - x_mean * y_mean
}

/// 一组三维样本的 3×3 协方差矩阵（对称）。
pub fn covariance_matrix(vec3: &[Vec3A]) -> Mat3A {
    let xs: Vec<f32> = vec3.iter().map(|v| v.x).collect();
    let ys: Vec<f32> = vec3.iter().map(|v| v.y).collect();
    let zs: Vec<f32> = vec3.iter().map(|v| v.z).collect();

    let xx = covariance(&xs, &xs);
    let xy = covariance(&xs, &ys);
    let xz = covariance(&xs, &zs);
    let yy = covariance(&ys, &ys);
    let yz = covariance(&ys, &zs);
    let zz = covariance(&zs, &zs);
    Mat3A {
        x_axis: Vec3A::new(xx, xy, xz),
        y_axis: Vec3A::new(xy, yy, yz),
        z_axis: Vec3A::new(xz, yz, zz),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::{Mat3A, Vec3A};

    #[test]
    fn test_self_outer_product_symmetry() {
        let v = Vec3A::new(1.0, 2.0, 3.0);
        let m = self_outer_product(v);
        // Check symmetry
        let cols = m.to_cols_array_2d();
        assert_eq!(cols[0][1], cols[1][0]);
        assert_eq!(cols[0][2], cols[2][0]);
        assert_eq!(cols[1][2], cols[2][1]);
    }

    #[test]
    fn test_self_outer_product_diagonal() {
        let v = Vec3A::new(2.0, 3.0, 4.0);
        let m = self_outer_product(v);
        assert_eq!(m.x_axis.x, 4.0); // 2*2
        assert_eq!(m.y_axis.y, 9.0); // 3*3
        assert_eq!(m.z_axis.z, 16.0); // 4*4
    }

    #[test]
    fn test_trace_of_product_identity() {
        let a = Mat3A::from_cols(
            Vec3A::new(1.0, 2.0, 3.0),
            Vec3A::new(4.0, 5.0, 6.0),
            Vec3A::new(7.0, 8.0, 9.0),
        );
        // Tr(I * A) = Tr(A) = a00 + a11 + a22
        let trace = trace_of_product(Mat3A::IDENTITY, a);
        assert!((trace - (1.0 + 5.0 + 9.0)).abs() < 1e-6);
    }

    #[test]
    fn test_variance_known_data() {
        let data = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let var = variance(&data);
        // mean = 5.0, variance = 4.0 (population)
        assert!((var - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_standard_deviation_known_data() {
        let data = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let sd = standard_deviation(&data);
        assert!((sd - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_covariance_identical() {
        let data = [1.0, 2.0, 3.0, 4.0, 5.0];
        let cov = covariance(&data, &data);
        let var = variance(&data);
        assert!((cov - var).abs() < 1e-5);
    }

    #[test]
    fn test_covariance_matrix_symmetry() {
        let vecs = [
            Vec3A::new(1.0, 2.0, 3.0),
            Vec3A::new(4.0, 5.0, 6.0),
            Vec3A::new(7.0, 8.0, 9.0),
        ];
        let m = covariance_matrix(&vecs);
        assert!((m.x_axis.y - m.y_axis.x).abs() < 1e-6);
        assert!((m.x_axis.z - m.z_axis.x).abs() < 1e-6);
        assert!((m.y_axis.z - m.z_axis.y).abs() < 1e-6);
    }

    #[test]
    fn test_cross_product_squared_transpose_symmetry() {
        let v = Vec3A::new(1.0, 2.0, 3.0);
        let m = cross_product_squared_transpose(v);
        let cols = m.to_cols_array_2d();
        assert!((cols[0][1] - cols[1][0]).abs() < 1e-6);
        assert!((cols[0][2] - cols[2][0]).abs() < 1e-6);
        assert!((cols[1][2] - cols[2][1]).abs() < 1e-6);
    }

    #[test]
    fn test_self_outer_product_values() {
        let m = self_outer_product(Vec3A::new(1.0, 2.0, 3.0)).to_cols_array_2d();
        let expected = [[1.0, 2.0, 3.0], [2.0, 4.0, 6.0], [3.0, 6.0, 9.0]];
        for (row, exp_row) in m.into_iter().zip(expected) {
            for (v, exp) in row.into_iter().zip(exp_row) {
                assert!((v - exp).abs() < 1e-6, "mismatch {v} vs expected {exp}");
            }
        }
    }

    #[test]
    fn test_trace_of_product_commutes() {
        let a = Mat3A::from_cols(
            Vec3A::new(1.0, 2.0, 3.0),
            Vec3A::new(4.0, 5.0, 6.0),
            Vec3A::new(7.0, 8.0, 9.0),
        );
        let b = Mat3A::from_cols(
            Vec3A::new(9.0, 8.0, 7.0),
            Vec3A::new(6.0, 5.0, 4.0),
            Vec3A::new(3.0, 2.0, 1.0),
        );
        // Tr(AB) = Tr(BA)；Tr(A·I) = Tr(A) = 1 + 5 + 9。
        let tr_ab = trace_of_product(a, b);
        let tr_ba = trace_of_product(b, a);
        assert!(
            (tr_ab - tr_ba).abs() < 1e-4,
            "Tr(AB)={tr_ab} Tr(BA)={tr_ba}"
        );
        let tr_ai = trace_of_product(a, Mat3A::IDENTITY);
        assert!((tr_ai - 15.0).abs() < 1e-4);
    }

    #[test]
    fn test_cross_interference_matrix_identity_pair() {
        // ci(I, I) = 2I。
        let m = cross_interference_matrix(Mat3A::IDENTITY, Mat3A::IDENTITY).to_cols_array_2d();
        for (i, row) in m.into_iter().enumerate() {
            for (j, v) in row.into_iter().enumerate() {
                let expected = if i == j { 2.0 } else { 0.0 };
                assert!((v - expected).abs() < 1e-6, "m[{i}][{j}]={v}");
            }
        }
    }

    #[test]
    fn test_cross_interference_matrix_zero() {
        assert_eq!(
            cross_interference_matrix(Mat3A::ZERO, Mat3A::IDENTITY),
            Mat3A::ZERO
        );
        assert_eq!(
            cross_interference_matrix(Mat3A::IDENTITY, Mat3A::ZERO),
            Mat3A::ZERO
        );
    }

    #[test]
    fn test_cross_interference_matrix_symmetry() {
        let p = Mat3A::from_cols(
            Vec3A::new(1.0, 0.5, 0.2),
            Vec3A::new(0.5, 2.0, 0.3),
            Vec3A::new(0.2, 0.3, 3.0),
        );
        let q = Mat3A::from_cols(
            Vec3A::new(2.0, 0.1, 0.4),
            Vec3A::new(0.1, 1.0, 0.6),
            Vec3A::new(0.4, 0.6, 0.5),
        );
        let cols = cross_interference_matrix(p, q).to_cols_array_2d();
        assert!((cols[0][1] - cols[1][0]).abs() < 1e-6);
        assert!((cols[0][2] - cols[2][0]).abs() < 1e-6);
        assert!((cols[1][2] - cols[2][1]).abs() < 1e-6);
    }

    #[test]
    fn test_first_order_tri_quad_identity_sigma() {
        // F(a, I) = a aᵀ − ||a||² I。
        let a = Vec3A::new(1.0, 2.0, 3.0);
        let m = first_order_tri_quad(a, Mat3A::IDENTITY).to_cols_array_2d();
        let norm2 = a.length_squared();
        let expected = [
            [1.0 - norm2, 2.0, 3.0],
            [2.0, 4.0 - norm2, 6.0],
            [3.0, 6.0, 9.0 - norm2],
        ];
        for (row, exp_row) in m.into_iter().zip(expected) {
            for (v, exp) in row.into_iter().zip(exp_row) {
                assert!((v - exp).abs() < 1e-4, "mismatch {v} vs expected {exp}");
            }
        }
    }

    #[test]
    fn test_first_order_tri_quad_zero_sigma() {
        let a = Vec3A::new(1.0, 2.0, 3.0);
        assert_eq!(first_order_tri_quad(a, Mat3A::ZERO), Mat3A::ZERO);
    }

    #[test]
    fn test_first_order_tri_quad_symmetry() {
        let a = Vec3A::new(0.3, -1.2, 2.0);
        let sigma = Mat3A::from_cols(
            Vec3A::new(1.0, 0.4, 0.2),
            Vec3A::new(0.4, 2.0, 0.1),
            Vec3A::new(0.2, 0.1, 3.0),
        );
        let cols = first_order_tri_quad(a, sigma).to_cols_array_2d();
        assert!((cols[0][1] - cols[1][0]).abs() < 1e-6);
        assert!((cols[0][2] - cols[2][0]).abs() < 1e-6);
        assert!((cols[1][2] - cols[2][1]).abs() < 1e-6);
    }

    #[test]
    fn test_cross_product_squared_transpose_identity() {
        // [v]ₓ[v]ₓᵀ = ||v||² I − v vᵀ。
        let v = Vec3A::new(1.0, 2.0, 3.0);
        let m = cross_product_squared_transpose(v).to_cols_array_2d();
        let norm2 = v.length_squared();
        let expected = [
            [norm2 - 1.0, -2.0, -3.0],
            [-2.0, norm2 - 4.0, -6.0],
            [-3.0, -6.0, norm2 - 9.0],
        ];
        for (row, exp_row) in m.into_iter().zip(expected) {
            for (v, exp) in row.into_iter().zip(exp_row) {
                assert!((v - exp).abs() < 1e-4, "mismatch {v} vs expected {exp}");
            }
        }
        // 单位基向量：diag(0,1,1)。
        let ex = cross_product_squared_transpose(Vec3A::X).to_cols_array_2d();
        assert!((ex[0][0] - 0.0).abs() < 1e-6);
        assert!((ex[1][1] - 1.0).abs() < 1e-6);
        assert!((ex[2][2] - 1.0).abs() < 1e-6);
        assert!((ex[0][1] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_variance_constant_and_single() {
        assert!((variance(&[3.0, 3.0, 3.0]) - 0.0).abs() < 1e-6);
        assert!((variance(&[5.0]) - 0.0).abs() < 1e-6);
        assert!((standard_deviation(&[5.0]) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_covariance_known_values() {
        let xs = [1.0, 2.0];
        let ys = [3.0, 4.0];
        // cov([1,2],[3,4]) = 0.25。
        assert!((covariance(&xs, &ys) - 0.25).abs() < 1e-6);
        // cov(x, x) = var(x)。
        assert!((covariance(&xs, &xs) - variance(&xs)).abs() < 1e-6);
        // 单元素：0。
        assert!((covariance(&[2.0], &[5.0]) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_covariance_matrix_known_values() {
        let vecs = [
            Vec3A::new(1.0, 0.0, 0.0),
            Vec3A::new(2.0, 0.0, 0.0),
            Vec3A::new(3.0, 0.0, 0.0),
        ];
        let m = covariance_matrix(&vecs).to_cols_array_2d();
        // var([1,2,3]) = 14/3 − 2² = 2/3，其余为 0。
        assert!((m[0][0] - 2.0 / 3.0).abs() < 1e-6, "m00={}", m[0][0]);
        for (i, row) in m.into_iter().enumerate() {
            for (j, v) in row.into_iter().enumerate() {
                if i != 0 || j != 0 {
                    assert!((v - 0.0).abs() < 1e-6, "m[{i}][{j}]={v}");
                }
            }
        }
    }

    #[test]
    fn test_covariance_matrix_constant() {
        let vecs = [Vec3A::new(1.0, 2.0, 3.0); 4];
        assert_eq!(covariance_matrix(&vecs), Mat3A::ZERO);
    }

    #[test]
    fn test_variance_empty_slice_is_nan() {
        let empty: &[f32] = &[];
        assert!(variance(empty).is_nan());
        assert!(standard_deviation(empty).is_nan());
    }
}
