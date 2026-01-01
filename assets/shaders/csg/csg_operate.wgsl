#define_import_path csg::csg_operate

fn op_round(primitive: f32, radius: f32) -> f32 {
    return primitive - radius;
}

fn op_union(d1: f32, d2: f32) -> f32 {
    return min(d1, d2);
}

fn op_subtraction(d1: f32, d2: f32) -> f32 {
    return max(-d1, d2);
}

fn op_intersection(d1: f32, d2: f32) -> f32 {
    return max(d1, d2);
}

fn op_subtraction_exact(d1: f32, d2: f32) -> f32 {
    return select(d2, -d1, d1 < 0.0);
}

fn op_smooth_union(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (d2 - d1) / k, 0.0, 1.0);
    return mix(d2, d1, h) - k * h * (1.0 - h);
}

fn op_smooth_subtraction(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 - 0.5 * (d2 + d1) / k, 0.0, 1.0);
    return mix(d2, -d1, h) + k * h * (1.0 - h);
}

fn op_smooth_intersection(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 - 0.5 * (d2 - d1) / k, 0.0, 1.0);
    return mix(d2, d1, h) + k * h * (1.0 - h);
}

