#define_import_path terrain::density_field
#import noise::fbm::open_simplex_2d_fbm_with_seed

#import terrain::csg::csg_utils::apply_csg_operations
#import terrain::main_mesh_bind_group::csg_operations

fn plane(location: vec3f, normal: vec3f, height: f32) -> f32 {
    // n must be normalized
    return dot(location, normal) + height;
}

fn cube(position: vec3f, half_size: vec3f) -> f32 {
    let q = abs(position) - half_size;
    return length(max(q, vec3f(0.0, 0.0, 0.0))) + min(max(max(q.x, q.y), q.z), 0.0);
}

const TERRAIN_HEIGHT_MAP_SIZE = 6;
var<private> TERRAIN_HEIGHT_MAP_INPUT: array<f32, TERRAIN_HEIGHT_MAP_SIZE> = array<f32, TERRAIN_HEIGHT_MAP_SIZE>(
    -1.0, -0.3, 0.0, 0.2, 0.4, 1.0
);
var<private> TERRAIN_HEIGHT_MAP_OUTPUT: array<f32, TERRAIN_HEIGHT_MAP_SIZE> = array<f32, TERRAIN_HEIGHT_MAP_SIZE>(
    -0.2, -0.1, 0.0, 0.05, 0.4, 1.0
);

// input is [-1.0, 1.0]
fn height_map(height: f32) -> f32 {
    if height < TERRAIN_HEIGHT_MAP_INPUT[0] {
        return TERRAIN_HEIGHT_MAP_INPUT[0];
    }

    for (var i = 0; i < TERRAIN_HEIGHT_MAP_SIZE - 1; i++) {
        if (TERRAIN_HEIGHT_MAP_INPUT[i] <= height) && (height < TERRAIN_HEIGHT_MAP_INPUT[i + 1]) {
            let t = (height - TERRAIN_HEIGHT_MAP_INPUT[i]) / (TERRAIN_HEIGHT_MAP_INPUT[i + 1] - TERRAIN_HEIGHT_MAP_INPUT[i]);
            return mix(TERRAIN_HEIGHT_MAP_OUTPUT[i], TERRAIN_HEIGHT_MAP_OUTPUT[i + 1], t);
        }
    }

    return TERRAIN_HEIGHT_MAP_INPUT[TERRAIN_HEIGHT_MAP_SIZE - 1];
}

fn get_terrain_noise(location: vec3f) -> f32 {
    let density_value = plane(location, vec3f(0.0, 1.0, 1.0), -1.0);
    // let loc = location + vec3f(6.0, 6.0, 6.0);
    // let density_value = cube(loc, vec3f(14.0, 14.0, 14.0));

    // let basic_noise = open_simplex_2d_fbm_with_seed(location.xz, 323u, 3u, 0.0025, 5.0, 3.0);
    // let mapped_height_value = height_map(basic_noise) * 100.0;

    // let density_value = location.y - mapped_height_value;

    return apply_csg_operations(location, density_value);
}
