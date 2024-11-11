#define_import_path math::map

fn map(v: f32, i_min: f32, i_max: f32, o_min: f32, o_max: f32) -> f32 {
    return o_min + (o_max - o_min) * (v - i_min) / (i_max - i_min);
}
fn map2(v: vec2f, i_min: vec2f, i_max: vec2f, o_min: vec2f, o_max: vec2f) -> vec2f {
    return o_min + (o_max - o_min) * (v - i_min) / (i_max - i_min);
}
fn map3(v: vec3f, i_min: vec3f, i_max: vec3f, o_min: vec3f, o_max: vec3f) -> vec3f {
    return o_min + (o_max - o_min) * (v - i_min) / (i_max - i_min);
}
fn map4(v: vec4f, i_min: vec4f, i_max: vec4f, o_min: vec4f, o_max: vec4f) -> vec4f {
    return o_min + (o_max - o_min) * (v - i_min) / (i_max - i_min);
}