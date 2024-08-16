use bevy::math::UVec4;

// range is [1, U32_MAX]
pub fn xorshift_128_with_seed(state: &mut UVec4) -> u32 {
    let st: UVec4 = *state;
    let mut t: u32 = st.w;
    let s: u32 = st.x;
    t ^= t << 11;
    t ^= t >> 8;
    let x: u32 = t ^ s ^ (s >> 19);
    *state = UVec4::new(x, s, st.y, st.z);
    x
}
