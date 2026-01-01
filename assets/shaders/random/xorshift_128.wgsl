#define_import_path random::xorshift_128

#import limit::numeric::U32_MAX

// The state must be initialized to non-zero
var<private> xorshift_128_state:vec4u = vec4u(1238973, 70890324, 768932, 987232398);

// range is [1, U32_MAX]
fn xorshift_128_with_seed(state: ptr<function, vec4u>) -> u32 {
    var st: vec4u = *state;
    // Algorithm "xor128" from p. 5 of Marsaglia, "Xorshift RNGs"
    // Load the state from the storage buffer
    var t: u32 = st.w;
    var s: u32 = st.x;
    t ^= t << 11;
    t ^= t >> 8;
    var x: u32 = t ^ s ^ (s >> 19);
    *state = vec4u(
        x, s, st.y, st.z
    );
    return x;
}

// range is [0, 1.0)
fn xorshift_128() -> f32 {
    return f32(xorshift_128_with_seed(&xorshift_128_state) - 1u) / f32(U32_MAX);
}

// range is [0, max)
fn xorshift_128_max(max: f32) -> f32 {
    return f32(xorshift_128_with_seed(&xorshift_128_state) - 1u) % max;
}

// range is [min, max)
fn xorshift_128_range(min: f32, max: f32) -> f32 {
    return mix(min, max, xorshift_128());
}

//// usage example

// fn my_main() {
//     var idx: u32 = /* index of this pixel into your rnd_state buffer * /;

//   // each pixel is assigned its own set of 4 random u32 for xorshift
//   // then this is stored in a local private space, so that each time
//   // random() is called, state will be shifted forward with xorshift
//   xorshift_128_state = vec4u(0, 1, 3, 4);
  
//   // now you are ready to call xorshift_128() as many times as you like!
//     out_color = vec4(
//         xorshift_128(),
//         xorshift_128(),
//         xorshift_128(),
//         1.0
//     );
// }