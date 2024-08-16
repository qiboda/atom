#define_import_path random::xorshift

#import limit::numeric::U32_MAX

var<private> xorshift_32_state: u32 = 80482034;

// The state must be initialized to non-zero
fn xorshift_32_with_seed(state: ptr<function, u32>) -> u32 {
	// Algorithm "xor" from p. 4 of Marsaglia, "Xorshift RNGs"
    var x = *state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *state = x;
    return x;
}

// range is [0.0, 1.0)
fn xorshift_32() -> f32 {
    return f32(xorshift_32_with_seed(&xorshift_32_state) - 1u) / f32(U32_MAX);
}

// range is [min, max)
fn xorshift_32_range(min: f32, max: f32) -> f32 {
    return mix(min, max, xorshift_32());
}