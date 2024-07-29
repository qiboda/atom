#define_import_path terrain::density_field

fn plane(location: vec3f, normal: vec3f, height: f32) -> f32 {
    // n must be normalized
    return dot(location, normal) + height;
}

fn cube(position: vec3f, half_size: vec3f) -> f32 {
    let q = abs(position) - half_size;
    return length(max(q, vec3f(0.0, 0.0, 0.0))) + min(max(max(q.x, q.y), q.z), 0.0);
}

fn get_terrain_noise(location: vec3f) -> f32 {
    return plane(location, vec3f(0.0, 1.0, 0.0), 2.0);
    // let loc = location + vec3f(6.0, 6.0, 6.0);
    // return cube(loc, vec3f(14.0, 14.0, 14.0));

}
