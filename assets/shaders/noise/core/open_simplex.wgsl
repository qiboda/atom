// TODO remove this if 
#define_import_path noise::open_simplex

const TABLE_SIZE: u32 = 256;

const FRAC_1_SQRT_2: f32 = 0.707106781186547524400844362104849039;
const DIAG: f32 = FRAC_1_SQRT_2;
const DIAG2: f32 = 0.5773502691896258;
const DIAG3: f32 = 0.5;

const STRETCH_CONSTANT_2D: f32 = -0.211324865405187; //(1/sqrt(2+1)-1)/2;
const SQUISH_CONSTANT_2D: f32 = 0.366025403784439; //(sqrt(2+1)-1)/2;
const NORM_CONSTANT_2D: f32 = 1.0 / 14.0;

const STRETCH_CONSTANT_3D: f32 = -1.0 / 6.0; //(1/Math.sqrt(3+1)-1)/3;
const SQUISH_CONSTANT_3D: f32 = 1.0 / 3.0; //(Math.sqrt(3+1)-1)/3;
const NORM_CONSTANT_3D: f32 = 1.0 / 14.0;

const STRETCH_CONSTANT_4D: f32 = -0.138196601125011; //(Math.sqrt(4+1)-1)/4;
const SQUISH_CONSTANT_4D: f32 = 0.309016994374947; //(Math.sqrt(4+1)-1)/4;
const NORM_CONSTANT_4D: f32 = 1.0 / 6.8699090070956625;


var<private> grad2:array<vec2f, 8> = array<vec2f, 8>(
    vec2f(1.0, 0.0),
    vec2f(-1.0, 0.0),
    vec2f(0.0, 1.0),
    vec2f(0.0, -1.0),
    vec2f(DIAG, DIAG),
    vec2f(-DIAG, DIAG),
    vec2f(DIAG, -DIAG),
    vec2f(-DIAG, -DIAG)
);

var<private> grad3:array<vec3f, 32> = array<vec3f, 32>(
    vec3f(DIAG, DIAG, 0.0),
    vec3f(-DIAG, DIAG, 0.0),
    vec3f(DIAG, -DIAG, 0.0),
    vec3f(-DIAG, -DIAG, 0.0),
    vec3f(DIAG, 0.0, DIAG),
    vec3f(-DIAG, 0.0, DIAG),
    vec3f(DIAG, 0.0, -DIAG),
    vec3f(-DIAG, 0.0, -DIAG),
    vec3f(0.0, DIAG, DIAG),
    vec3f(0.0, -DIAG, DIAG),
    vec3f(0.0, DIAG, -DIAG),
    vec3f(0.0, -DIAG, -DIAG),
    vec3f(DIAG, DIAG, 0.0),
    vec3f(-DIAG, DIAG, 0.0),
    vec3f(DIAG, -DIAG, 0.0),
    vec3f(-DIAG, -DIAG, 0.0),
    vec3f(DIAG, 0.0, DIAG),
    vec3f(-DIAG, 0.0, DIAG),
    vec3f(DIAG, 0.0, -DIAG),
    vec3f(-DIAG, 0.0, -DIAG),
    vec3f(0.0, DIAG, DIAG),
    vec3f(0.0, -DIAG, DIAG),
    vec3f(0.0, DIAG, -DIAG),
    vec3f(0.0, -DIAG, -DIAG),
    vec3f(DIAG2, DIAG2, DIAG2),
    vec3f(-DIAG2, DIAG2, DIAG2),
    vec3f(DIAG2, -DIAG2, DIAG2),
    vec3f(-DIAG2, -DIAG2, DIAG2),
    vec3f(DIAG2, DIAG2, -DIAG2),
    vec3f(-DIAG2, DIAG2, -DIAG2),
    vec3f(DIAG2, -DIAG2, -DIAG2),
    vec3f(-DIAG2, -DIAG2, -DIAG2),
);

var<private> grad4:array<vec4f, 64> = array<vec4f, 64>(
    vec4f(0.0, DIAG2, DIAG2, DIAG2),
    vec4f(0.0, DIAG2, DIAG2, -DIAG2),
    vec4f(0.0, DIAG2, -DIAG2, DIAG2),
    vec4f(0.0, DIAG2, -DIAG2, -DIAG2),
    vec4f(0.0, -DIAG2, DIAG2, DIAG2),
    vec4f(0.0, -DIAG2, DIAG2, -DIAG2),
    vec4f(0.0, -DIAG2, -DIAG2, DIAG2),
    vec4f(0.0, -DIAG2, -DIAG2, -DIAG2),
    vec4f(DIAG2, 0.0, DIAG2, DIAG2),
    vec4f(DIAG2, 0.0, DIAG2, -DIAG2),
    vec4f(DIAG2, 0.0, -DIAG2, DIAG2),
    vec4f(DIAG2, 0.0, -DIAG2, -DIAG2),
    vec4f(-DIAG2, 0.0, DIAG2, DIAG2),
    vec4f(-DIAG2, 0.0, DIAG2, -DIAG2),
    vec4f(-DIAG2, 0.0, -DIAG2, DIAG2),
    vec4f(-DIAG2, 0.0, -DIAG2, -DIAG2),
    vec4f(DIAG2, DIAG2, 0.0, DIAG2),
    vec4f(DIAG2, DIAG2, 0.0, -DIAG2),
    vec4f(DIAG2, -DIAG2, 0.0, DIAG2),
    vec4f(DIAG2, -DIAG2, 0.0, -DIAG2),
    vec4f(-DIAG2, DIAG2, 0.0, DIAG2),
    vec4f(-DIAG2, DIAG2, 0.0, -DIAG2),
    vec4f(-DIAG2, -DIAG2, 0.0, DIAG2),
    vec4f(-DIAG2, -DIAG2, 0.0, -DIAG2),
    vec4f(DIAG2, DIAG2, DIAG2, 0.0),
    vec4f(DIAG2, DIAG2, -DIAG2, 0.0),
    vec4f(DIAG2, -DIAG2, DIAG2, 0.0),
    vec4f(DIAG2, -DIAG2, -DIAG2, 0.0),
    vec4f(-DIAG2, DIAG2, DIAG2, 0.0),
    vec4f(-DIAG2, DIAG2, -DIAG2, 0.0),
    vec4f(-DIAG2, -DIAG2, DIAG2, 0.0),
    vec4f(-DIAG2, -DIAG2, -DIAG2, 0.0),
    vec4f(DIAG3, DIAG3, DIAG3, DIAG3),
    vec4f(-DIAG3, DIAG3, DIAG3, DIAG3),
    vec4f(DIAG3, -DIAG3, DIAG3, DIAG3),
    vec4f(-DIAG3, -DIAG3, DIAG3, DIAG3),
    vec4f(DIAG3, DIAG3, -DIAG3, DIAG3),
    vec4f(-DIAG3, DIAG3, -DIAG3, DIAG3),
    vec4f(DIAG3, DIAG3, DIAG3, -DIAG3),
    vec4f(-DIAG3, DIAG3, DIAG3, -DIAG3),
    vec4f(DIAG3, -DIAG3, -DIAG3, DIAG3),
    vec4f(-DIAG3, -DIAG3, -DIAG3, DIAG3),
    vec4f(DIAG3, -DIAG3, DIAG3, -DIAG3),
    vec4f(-DIAG3, -DIAG3, DIAG3, -DIAG3),
    vec4f(DIAG3, DIAG3, -DIAG3, -DIAG3),
    vec4f(-DIAG3, DIAG3, -DIAG3, -DIAG3),
    vec4f(DIAG3, -DIAG3, -DIAG3, -DIAG3),
    vec4f(-DIAG3, -DIAG3, -DIAG3, -DIAG3),
    vec4f(DIAG3, DIAG3, DIAG3, DIAG3),
    vec4f(-DIAG3, DIAG3, DIAG3, DIAG3),
    vec4f(DIAG3, -DIAG3, DIAG3, DIAG3),
    vec4f(-DIAG3, -DIAG3, DIAG3, DIAG3),
    vec4f(DIAG3, DIAG3, -DIAG3, DIAG3),
    vec4f(-DIAG3, DIAG3, -DIAG3, DIAG3),
    vec4f(DIAG3, DIAG3, DIAG3, -DIAG3),
    vec4f(-DIAG3, DIAG3, DIAG3, -DIAG3),
    vec4f(DIAG3, -DIAG3, -DIAG3, DIAG3),
    vec4f(-DIAG3, -DIAG3, -DIAG3, DIAG3),
    vec4f(DIAG3, -DIAG3, DIAG3, -DIAG3),
    vec4f(-DIAG3, -DIAG3, DIAG3, -DIAG3),
    vec4f(DIAG3, DIAG3, -DIAG3, -DIAG3),
    vec4f(-DIAG3, DIAG3, -DIAG3, -DIAG3),
    vec4f(DIAG3, -DIAG3, -DIAG3, -DIAG3),
    vec4f(-DIAG3, -DIAG3, -DIAG3, -DIAG3),
);

fn hash(permutation_table: ptr<function, array<u32, TABLE_SIZE>>, to_hash: i32) -> u32 {
    let index = to_hash & 0xff;
    return (*permutation_table)[index];
}

fn hash_21(permutation_table: ptr<function, array<u32, TABLE_SIZE>>, to_hash: vec2i) -> u32 {
    let index_a = to_hash.x & 0xff;
    let b = to_hash.y & 0xff;
    let index_b = (*permutation_table)[index_a] ^ u32(b);
    return (*permutation_table)[index_b];
}

fn hash_31(permutation_table: ptr<function, array<u32, TABLE_SIZE>>, to_hash: vec3i) -> u32 {
    let index_a = to_hash.x & 0xff;
    let b = to_hash.y & 0xff;
    let c = to_hash.z & 0xff;
    let index_b = (*permutation_table)[index_a] ^ u32(b);
    let index_c = (*permutation_table)[index_b] ^ u32(c);
    return (*permutation_table)[index_c];
}

fn hash_41(permutation_table: ptr<function, array<u32, TABLE_SIZE>>, to_hash: vec4i) -> u32 {
    let index_a = to_hash.x & 0xff;
    let b = to_hash.y & 0xff;
    let c = to_hash.z & 0xff;
    let d = to_hash.w & 0xff;
    let index_b = (*permutation_table)[index_a] ^ u32(b);
    let index_c = (*permutation_table)[index_b] ^ u32(c);
    let index_d = (*permutation_table)[index_c] ^ u32(d);
    return (*permutation_table)[index_d];
}

fn surflet_2d(index: u32, point: vec2f) -> f32 {
    // magnitude_squared
    let t = 2.0 - dot(point, point);

    if t > 0.0 {
        let gradient = grad2[index % 8u];
        return pow(t, 4.0) * dot(point, gradient);
    } else {
        return 0.0;
    }
}

fn contribute_2d(
    permutation_table: ptr<function, array<u32, TABLE_SIZE>>,
    stretched_floor: vec2f,
    rel_pos: vec2f,
    x: f32, y: f32
) -> f32 {
    let offset = vec2(x, y);
    let vertex = stretched_floor + offset;
    let index = hash_21(permutation_table, vec2i(vertex));
    let dpos = rel_pos - (SQUISH_CONSTANT_2D * (offset.x + offset.y)) - offset;

    return surflet_2d(index, dpos);
}

// out range: [-1, 1]
fn open_simplex_2d(point: vec2f, permutation_table: ptr<function, array<u32, TABLE_SIZE>>) -> f32 {

    // Place input coordinates onto grid.
    let stretch_offset = (point.x + point.y) * STRETCH_CONSTANT_2D;
    let stretched = point + vec2f(stretch_offset);

    // Floor to get grid coordinates of rhombus (stretched square) cell origin.
    // TODO: modf 暂时不支持。
    // let modf_stretched = modf(stretched);
    let stretched_floor = floor(stretched);
    // let stretched_floor = modf_stretched.whole;
    // Compute grid coordinates relative to rhombus origin.
    let rel_coords = stretched - stretched_floor;
    // let rel_coords = modf_stretched.fract;

    // Skew out to get actual coordinates of rhombus origin. We'll need these later.
    let squish_offset = (stretched_floor.x + stretched_floor.y) * SQUISH_CONSTANT_2D;
    let origin = stretched_floor + vec2(squish_offset);


    // Sum those together to get a value that determines which region we're in.
    let region_sum = rel_coords.x + rel_coords.y;

    // Positions relative to origin point (0, 0).
    let rel_pos = point - origin;

    var value = 0.0;

    // (0, 0) --- (1, 0)
    // |   A     /     |
    // |       /       |
    // |     /     B   |
    // (0, 1) --- (1, 1)

    // Contribution (1, 0)
    value += contribute_2d(permutation_table, stretched_floor, rel_pos, 1.0, 0.0);

    // Contribution (0, 1)
    value += contribute_2d(permutation_table, stretched_floor, rel_pos, 0.0, 1.0);

    // See the graph for an intuitive explanation; the sum of `x` and `y` is
    // only greater than `1` if we're on Region B.
    if region_sum > 1.0 {
        // Contribution (1, 1)
        value += contribute_2d(permutation_table, stretched_floor, rel_pos, 1.0, 1.0);
    } else {
        // Contribution (1, 1)
        value += contribute_2d(permutation_table, stretched_floor, rel_pos, 0.0, 0.0);
    }

    return value * NORM_CONSTANT_2D;
}

fn surflet_3d(index: u32, point: vec3f) -> f32 {
    // magnitude_squared
    let t = 2.0 - dot(point, point);

    if t > 0.0 {
        let gradient = grad3[index % 32];
        return pow(t, 4.0) * dot(point, gradient);
    } else {
        return 0.0;
    }
}

fn contribute_3d(
    permutation_table: ptr<function, array<u32, TABLE_SIZE>>,
    stretched_floor: vec3f,
    rel_pos: vec3f,
    x: f32, y: f32, z: f32
) -> f32 {
    let offset = vec3f(x, y, z);
    let vertex = stretched_floor + offset;
    let index = hash_31(permutation_table, vec3i(vertex));
    let dpos = rel_pos - (SQUISH_CONSTANT_3D * (offset.x + offset.y + offset.z)) - offset;
    return surflet_3d(index, dpos);
}

// out range: [-1, 1]
fn open_simplex_3d(point: vec3f, permutation_table: ptr<function, array<u32, TABLE_SIZE>>) -> f32 {

    // Place input coordinates on simplectic honeycomb.
    let stretch_offset = (point.x + point.y + point.z) * STRETCH_CONSTANT_3D;
    let stretched = point + vec3f(stretch_offset);

    // Floor to get simplectic honeycomb coordinates of rhombohedron
    // (stretched cube) super-cell origin.
    // let modf_stretched = modf(stretched);
    let stretched_floor = floor(stretched);
    // let stretched_floor = modf_stretched.whole;
    // Compute simplectic honeycomb coordinates relative to rhombohedral origin.
    let rel_coords = stretched - stretched_floor;
    // let rel_coords = modf_stretched.fract;

    // Skew out to get actual coordinates of rhombohedron origin. We'll need
    // these later.
    let squish_offset = (stretched_floor.x + stretched_floor.y + stretched_floor.z) * SQUISH_CONSTANT_3D;
    let origin = stretched_floor + vec3f(squish_offset);


    // Sum those together to get a value that determines which region we're in.
    let region_sum = rel_coords.x + rel_coords.y + rel_coords.z;

    // Positions relative to origin point.
    let rel_pos = point - origin;

    var value = 0.0;

    if region_sum <= 1.0 {
        // We're inside the tetrahedron (3-Simplex) at (0, 0, 0)

        // Contribution at (0, 0, 0)
        value += contribute_3d(permutation_table, stretched_floor, rel_pos, 0.0, 0.0, 0.0);

        // Contribution at (1, 0, 0)
        value += contribute_3d(permutation_table, stretched_floor, rel_pos, 1.0, 0.0, 0.0);

        // Contribution at (0, 1, 0)
        value += contribute_3d(permutation_table, stretched_floor, rel_pos, 0.0, 1.0, 0.0);

        // Contribution at (0, 0, 1)
        value += contribute_3d(permutation_table, stretched_floor, rel_pos, 0.0, 0.0, 1.0);
    } else if region_sum >= 2.0 {
        // We're inside the tetrahedron (3-Simplex) at (1, 1, 1)

        // Contribution at (1, 1, 0)
        value += contribute_3d(permutation_table, stretched_floor, rel_pos, 1.0, 1.0, 0.0);

        // Contribution at (1, 0, 1)
        value += contribute_3d(permutation_table, stretched_floor, rel_pos, 1.0, 0.0, 1.0);

        // Contribution at (0, 1, 1)
        value += contribute_3d(permutation_table, stretched_floor, rel_pos, 0.0, 1.0, 1.0);

        // Contribution at (1, 1, 1)
        value += contribute_3d(permutation_table, stretched_floor, rel_pos, 1.0, 1.0, 1.0);
    } else {
        // We're inside the octahedron (Rectified 3-Simplex) inbetween.

        // Contribution at (1, 0, 0)
        value += contribute_3d(permutation_table, stretched_floor, rel_pos, 1.0, 0.0, 0.0);

        // Contribution at (0, 1, 0)
        value += contribute_3d(permutation_table, stretched_floor, rel_pos, 0.0, 1.0, 0.0);

        // Contribution at (0, 0, 1)
        value += contribute_3d(permutation_table, stretched_floor, rel_pos, 0.0, 0.0, 1.0);

        // Contribution at (1, 1, 0)
        value += contribute_3d(permutation_table, stretched_floor, rel_pos, 1.0, 1.0, 0.0);

        // Contribution at (1, 0, 1)
        value += contribute_3d(permutation_table, stretched_floor, rel_pos, 1.0, 0.0, 1.0);

        // Contribution at (0, 1, 1)
        value += contribute_3d(permutation_table, stretched_floor, rel_pos, 0.0, 1.0, 1.0);
    }

    return value * NORM_CONSTANT_3D;
}

fn surflet_4d(index: u32, point: vec4f) -> f32 {
    // let t = 2.0 - point.magnitude_squared();
    let t = 2.0 - dot(point, point);

    if t > 0.0 {
        let gradient = grad4[index % 64];
        return pow(t, 4.0) * dot(point, gradient);
    } else {
        return 0.0;
    }
}


fn contribute_4d(
    permutation_table: ptr<function, array<u32, TABLE_SIZE>>,
    stretched_floor: vec4f,
    rel_pos: vec4f,
    x: f32, y: f32, z: f32, w: f32
) -> f32 {
    let offset = vec4f(x, y, z, w);
    let vertex = stretched_floor + offset;
    let index = hash_41(permutation_table, vec4i(vertex));
    let dpos = rel_pos - (SQUISH_CONSTANT_4D * (offset.x + offset.y + offset.z + offset.w)) - offset;
    return surflet_4d(index, dpos);
}

// out range: [-1, 1]
fn open_simplex_4d(point: vec4f, permutation_table: ptr<function, array<u32, TABLE_SIZE>>) -> f32 {
    // Place input coordinates on simplectic honeycomb.
    let stretch_offset = (point.x + point.y + point.z + point.w) * STRETCH_CONSTANT_4D;
    let stretched = point + stretch_offset;

    // Floor to get simplectic honeycomb coordinates of rhombo-hypercube
    // super-cell origin.
    // let stretched_modf = modf(stretched);
    let stretched_floor = floor(stretched);
    // let stretched_floor = stretched_modf.whole;
    let rel_coords = stretched - stretched_floor;
    // let rel_coords = stretched_modf.fract;

    // Skew out to get actual coordinates of stretched rhombo-hypercube origin.
    // We'll need these later.
    let squish_offset = (stretched_floor.x + stretched_floor.y + stretched_floor.z + stretched_floor.w) * SQUISH_CONSTANT_4D;
    let origin = stretched_floor + vec4f(squish_offset);

    // Compute simplectic honeycomb coordinates relative to rhombo-hypercube
    // origin.

    // Sum those together to get a value that determines which region
    // we're in.
    let region_sum = (rel_coords.x + rel_coords.y + rel_coords.z + rel_coords.w);

    // Position relative to origin point.
    let rel_pos = point - origin;

    var value = 0.0;

    if region_sum <= 1.0 {
        // We're inside the pentachoron (4-Simplex) at (0, 0, 0, 0)

        // Contribution at (0, 0, 0, 0)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 0.0, 0.0, 0.0, 0.0);

        // Contribution at (1, 0, 0, 0)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 1.0, 0.0, 0.0, 0.0);

        // Contribution at (0, 1, 0, 0)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 0.0, 1.0, 0.0, 0.0);

        // Contribution at (0, 0, 1, 0)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 0.0, 0.0, 1.0, 0.0);

        // Contribution at (0, 0, 0, 1)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 0.0, 0.0, 0.0, 1.0);
    } else if region_sum >= 3.0 {
        // We're inside the pentachoron (4-Simplex) at (1, 1, 1, 1)

        // Contribution at (1, 1, 1, 0)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 1.0, 1.0, 1.0, 0.0);

        // Contribution at (1, 1, 0, 1)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 1.0, 1.0, 0.0, 1.0);

        // Contribution at (1, 0, 1, 1)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 1.0, 0.0, 1.0, 1.0);

        // Contribution at (0, 1, 1, 1)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 0.0, 1.0, 1.0, 1.0);

        // Contribution at (1, 1, 1, 1)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 1.0, 1.0, 1.0, 1.0);
    } else if region_sum <= 2.0 {
        // We're inside the first dispentachoron (Rectified 4-Simplex)

        // Contribution at (1, 0, 0, 0)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 1.0, 0.0, 0.0, 0.0);

        // Contribution at (0, 1, 0, 0)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 0.0, 1.0, 0.0, 0.0);

        // Contribution at (0, 0, 1, 0)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 0.0, 0.0, 1.0, 0.0);

        // Contribution at (0, 0, 0, 1)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 0.0, 0.0, 0.0, 1.0);

        // Contribution at (1, 1, 0, 0)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 1.0, 1.0, 0.0, 0.0);

        // Contribution at (1, 0, 1, 0)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 1.0, 0.0, 1.0, 0.0);

        // Contribution at (1, 0, 0, 1)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 1.0, 0.0, 0.0, 1.0);

        // Contribution at (0, 1, 1, 0)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 0.0, 1.0, 1.0, 0.0);

        // Contribution at (0, 1, 0, 1)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 0.0, 1.0, 0.0, 1.0);

        // Contribution at (0, 0, 1, 1)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 0.0, 0.0, 1.0, 1.0);
    } else {
        // We're inside the second dispentachoron (Rectified 4-Simplex)

        // Contribution at (1, 1, 1, 0)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 1.0, 1.0, 1.0, 0.0);

        // Contribution at (1, 1, 0, 1)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 1.0, 1.0, 0.0, 1.0);

        // Contribution at (1, 0, 1, 1)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 1.0, 0.0, 1.0, 1.0);

        // Contribution at (0, 1, 1, 1)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 0.0, 1.0, 1.0, 1.0);

        // Contribution at (1, 1, 0, 0)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 1.0, 1.0, 0.0, 0.0);

        // Contribution at (1, 0, 1, 0)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 1.0, 0.0, 1.0, 0.0);

        // Contribution at (1, 0, 0, 1)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 1.0, 0.0, 0.0, 1.0);

        // Contribution at (0, 1, 1, 0)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 0.0, 1.0, 1.0, 0.0);

        // Contribution at (0, 1, 0, 1)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 0.0, 1.0, 0.0, 1.0);

        // Contribution at (0, 0, 1, 1)
        value += contribute_4d(permutation_table, stretched_floor, rel_pos, 0.0, 0.0, 1.0, 1.0);
    }

    return value * NORM_CONSTANT_4D;
}

