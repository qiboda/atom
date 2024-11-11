use bevy::math::{IVec2, Vec2};

use super::open_simplex_seed::TABLE_SIZE;

#[allow(clippy::excessive_precision)]
const FRAC_1_SQRT_2: f32 = 0.707106781186547524400844362104849039;
const DIAG: f32 = FRAC_1_SQRT_2;

#[allow(clippy::excessive_precision)]
const STRETCH_CONSTANT_2D: f32 = -0.211324865405187; //(1/sqrt(2+1)-1)/2;
#[allow(clippy::excessive_precision)]
const SQUISH_CONSTANT_2D: f32 = 0.366025403784439; //(sqrt(2+1)-1)/2;
const NORM_CONSTANT_2D: f32 = 1.0 / 14.0;

const GRAD2: [Vec2; 8] = [
    Vec2::new(1.0, 0.0),
    Vec2::new(-1.0, 0.0),
    Vec2::new(0.0, 1.0),
    Vec2::new(0.0, -1.0),
    Vec2::new(DIAG, DIAG),
    Vec2::new(-DIAG, DIAG),
    Vec2::new(DIAG, -DIAG),
    Vec2::new(-DIAG, -DIAG),
];

fn hash_21(permutation_table: [u32; TABLE_SIZE], to_hash: IVec2) -> u32 {
    let index_a = to_hash.x & 0xff;
    let b = to_hash.y & 0xff;
    let index_b = permutation_table[index_a as usize] as usize ^ b as usize;

    // if to_hash.x == -209 && to_hash.y == 788 {
    //     println!("index_a: {}, b: {}, index_b: {}", index_a, b, index_b);
    // }
    permutation_table[index_b]
}

fn surflet_2d(index: usize, point: Vec2) -> f32 {
    // magnitude_squared
    let t = 2.0 - point.dot(point);

    if t > 0.0 {
        let gradient = GRAD2[index % 8];
        t.powi(4) * point.dot(gradient)
    } else {
        0.0
    }
}

fn contribute_2d(
    point: Vec2,
    permutation_table: [u32; TABLE_SIZE],
    stretched_floor: Vec2,
    rel_pos: Vec2,
    x: f32,
    y: f32,
) -> f32 {
    let offset = Vec2::new(x, y);
    let vertex = stretched_floor + offset;
    let index = hash_21(
        permutation_table,
        IVec2::new(vertex.x as i32, vertex.y as i32),
    );
    let dpos = rel_pos - (SQUISH_CONSTANT_2D * (offset.x + offset.y)) - offset;

    if point.x == 2.0 && point.y == 1000.0 {
        println!("vertex: {}", vertex);
        println!("index: {}, dpos: {}", index, dpos);
    }
    surflet_2d(index as usize, dpos)
}

// out range: [-1, 1]
#[allow(dead_code)]
pub fn open_simplex_2d(point: Vec2, permutation_table: [u32; TABLE_SIZE]) -> f32 {
    // Place input coordinates onto grid.
    let stretch_offset = (point.x + point.y) * STRETCH_CONSTANT_2D;
    let stretched = point + Vec2::splat(stretch_offset);

    if point.x == 2.0 && point.y == 1000.0 {
        println!("stretched: {}", stretched);
    }

    // Floor to get grid coordinates of rhombus (stretched square) cell origin.
    // TODO: modf 暂时不支持。
    // let modf_stretched = modf(stretched);
    let stretched_floor = stretched.floor();
    if point.x == 2.0 && point.y == 1000.0 {
        println!("stretched_floor: {}", stretched_floor);
    }
    // let stretched_floor = modf_stretched.whole;
    // Compute grid coordinates relative to rhombus origin.
    let rel_coords = stretched - stretched_floor;
    // let rel_coords = modf_stretched.fract;
    if point.x == 2.0 && point.y == 1000.0 {
        println!("rel_coords: {}", rel_coords);
    }

    // Skew out to get actual coordinates of rhombus origin. We'll need these later.
    let squish_offset = (stretched_floor.x + stretched_floor.y) * SQUISH_CONSTANT_2D;
    let origin = stretched_floor + Vec2::splat(squish_offset);
    if point.x == 2.0 && point.y == 1000.0 {
        println!("origin: {}", origin);
    }

    // Sum those together to get a value that determines which region we're in.
    let region_sum = rel_coords.x + rel_coords.y;
    if point.x == 2.0 && point.y == 1000.0 {
        println!("region_sum: {}", region_sum);
    }

    // Positions relative to origin point (0, 0).
    let rel_pos = point - origin;
    if point.x == 2.0 && point.y == 1000.0 {
        println!("rel_pos: {}", rel_pos);
    }

    let mut value = 0.0;

    // (0, 0) --- (1, 0)
    // |   A     /     |
    // |       /       |
    // |     /     B   |
    // (0, 1) --- (1, 1)

    // Contribution (1, 0)
    value += contribute_2d(point, permutation_table, stretched_floor, rel_pos, 1.0, 0.0);

    if point.x == 2.0 && point.y == 1000.0 {
        println!("value: {}", value);
    }

    // Contribution (0, 1)
    value += contribute_2d(point, permutation_table, stretched_floor, rel_pos, 0.0, 1.0);

    if point.x == 2.0 && point.y == 1000.0 {
        println!("value: {}", value);
    }

    // See the graph for an intuitive explanation; the sum of `x` and `y` is
    // only greater than `1` if we're on Region B.
    if region_sum > 1.0 {
        // Contribution (1, 1)
        value += contribute_2d(point, permutation_table, stretched_floor, rel_pos, 1.0, 1.0);

        if point.x == 2.0 && point.y == 1000.0 {
            println!("value > 1.0: {}", value);
        }
    } else {
        // Contribution (1, 1)
        value += contribute_2d(point, permutation_table, stretched_floor, rel_pos, 0.0, 0.0);

        if point.x == 2.0 && point.y == 1000.0 {
            println!("value <= 1.0: {}", value);
        }
    }

    value * NORM_CONSTANT_2D
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use bevy::math::Vec2;

    use crate::gpu_test::open_simplex_seed::generate_permutation_table;

    use super::open_simplex_2d;

    #[test]
    fn test_simplex() {
        let mut pixels: Vec<u8> = Vec::with_capacity(1024 * 1024);

        let permutation_table = generate_permutation_table(233233223);
        println!("{:?}", permutation_table);
        let stride = 1024.0 / 1024.0;
        for i in 0..1024 {
            for j in 0..1024 {
                let value = open_simplex_2d(
                    Vec2::new(stride * i as f32, stride * j as f32),
                    permutation_table,
                );
                let value = (value + 1.0) * 0.5;
                pixels.push((value * 255.0) as u8);
            }
        }

        let _ = image::save_buffer(
            Path::new("open_simplex.png"),
            &pixels,
            1024,
            1024,
            image::ColorType::L8,
        );
    }
}
