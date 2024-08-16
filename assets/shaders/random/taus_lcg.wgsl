// 高质量的随机函数
// https://developer.nvidia.com/gpugems/GPUGems3/gpugems3_ch37.html
// https://math.stackexchange.com/questions/337782/pseudo-random-number-generation-on-the-gpu
#define_import_path random::taus_lcg

var<private> taus_lcg_state: vec4u = vec4u(123899073, 7090324, 768990932, 8732398);

fn taus_step(z: u32, S1: i32, S2: i32, S3: i32, M: u32) -> u32 {
    uint b = (((z << S1) ^ z) >> S2);
    return (((z & M) << S3) ^ b);
}

fn lcg_step(z: u32, A: u32, C: u32) -> u32 {
    return (A * z + C);
}

// Note that the initial seed values for state should be larger than 128! 
// (For background information reed the paper) 
// and that you should fill the seed with 4 good random numbers from the CPU 
// + four values unique for that pixel to get a nice result.
// range is [0.0, 1.0]
fn taus_lcg_with_seed(state: ptr<function, vec4u>) -> f32 {
    (*state).x = taus_step((*state).x, 13, 19, 12, 4294967294u);
    (*state).y = taus_step((*state).y, 2, 25, 4, 4294967288u);
    (*state).z = taus_step((*state).z, 3, 11, 17, 4294967280u);
    (*state).w = lcg_step((*state).w, 1664525u, 1013904223u);
    return 2.3283064365387e-10 * (state.x ^ state.y ^ state.z ^ state.w);
}

// range is [0.0, 1.0]
fn taus_lcg() -> f32 {
    return taus_lcg_with_seed(&taus_lcg_state);
}

// range is [min, max]
fn taus_lcg_range(min: f32, max: f32) -> f32 {
    return mix(min, max, taus_lcg());
}
