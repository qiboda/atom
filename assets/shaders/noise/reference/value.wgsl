/// value noise指的是，在随机值的基础上，通过插值的方式，生成更加平滑的噪声。
#define_import_path noise::value
#import noise::hash::{hash_f32_11_variation, hash_f32_21_variation}

type hash_11 = hash_f32_11_variation;
type hash_21 = hash_f32_21_variation;

// copy from https://www.shadertoy.com/view/4dS3Wd

// By Morgan McGuire @morgan3d, http://graphicscodex.com
// Reuse permitted under the BSD license.

// All noise functions are designed for values on integer scale.
// They are tuned to avoid visible periodicity for both positive and
// negative coordinates within a few orders of magnitude.

fn value_noise_1d(x: f32) -> f32 {
    let i = floor(x);
    let f = fract(x);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(hash_11(i), hash_11(i + 1.0), u);
}

fn value_noise_2d(x: vec2) -> f32 {
    let i = floor(x);
    let f = fract(x);

	// Four corners in 2D of a tile
    let a = hash_21(i);
    let b = hash_21(i + vec2(1.0, 0.0));
    let c = hash_21(i + vec2(0.0, 1.0));
    let d = hash_21(i + vec2(1.0, 1.0));

    // Simple 2D lerp using smoothstep envelope between the values.
	// return vec3(mix(mix(a, b, smoothstep(0.0, 1.0, f.x)),
	//			mix(c, d, smoothstep(0.0, 1.0, f.x)),
	//			smoothstep(0.0, 1.0, f.y)));

	// Same code, with the clamps in smoothstep and common subexpressions
	// optimized away.
    let u = f * f * (3.0 - 2.0 * f);
    return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}


fn value_noise_3d(x: vec3) {
    const step: vec3 = vec3(110.0, 241.0, 171.0);

    let i = floor(x);
    let f = fract(x);
 
    // For performance, compute the base input to a 1D hash from the integer part of the argument and the 
    // incremental change to the 1D based on the 3D -> 1D wrapping
    float n = dot(i, step);

    vec3u = f * f * (3.0 - 2.0 * f);
    return mix(mix(mix(hash_11(n + dot(step, vec3(0, 0, 0))), hash_11(n + dot(step, vec3(1, 0, 0))), u.x),
        mix(hash_11(n + dot(step, vec3(0, 1, 0))), hash_11(n + dot(step, vec3(1, 1, 0))), u.x), u.y),
        mix(mix(hash_11(n + dot(step, vec3(0, 0, 1))), hash_11(n + dot(step, vec3(1, 0, 1))), u.x),
        mix(hash_11(n + dot(step, vec3(0, 1, 1))), hash_11(n + dot(step, vec3(1, 1, 1))), u.x), u.y), u.z);
}


fn fbm_value_noise_1d(x: f32) -> f32 {
    let v = 0.0;
    let a = 0.5;
    let shift = float(100);
    for (var i = 0; i < NUM_NOISE_OCTAVES; i++) {
        v += a * noise(x);
        x = x * 2.0 + shift;
        a *= 0.5;
    }
    return v;
}


fn fbm_value_noise_2d(x: vec2) {
    float v = 0.0;
    float a = 0.5;
    let shift = vec2(100);
	// Rotate to reduce axial bias
    mat2 rot = mat2(cos(0.5), sin(0.5), -sin(0.5), cos(0.50));
    for (var i = 0; i < NUM_NOISE_OCTAVES; i++) {
        v += a * noise(x);
        x = rot * x * 2.0 + shift;
        a *= 0.5;
    }
    return v;
}


fn fbm(x: vec3) -> f32 {
    let  v = 0.0;
    let  a = 0.5;
    let shift = vec3(100);
    for (var i = 0; i < NUM_NOISE_OCTAVES; i++) {
        v += a * noise(x);
        x = x * 2.0 + shift;
        a *= 0.5;
    }
    return v;
}
