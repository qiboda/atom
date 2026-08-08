use bevy::math::UVec4;

// range is [1, U32_MAX]
/// 基于 xorshift128 算法从 128 位状态中生成一个伪随机数，与 shader 中的 `xorshift_128` 对齐。
///
/// 原地更新 `state`（4 个 `u32` 组成），返回值为 `[1, U32_MAX]` 区间内的随机数。
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::UVec4;

    #[test]
    fn test_deterministic() {
        let mut state1 = UVec4::new(1, 42, 42, 42);
        let mut state2 = UVec4::new(1, 42, 42, 42);
        let a = xorshift_128_with_seed(&mut state1);
        let b = xorshift_128_with_seed(&mut state2);
        assert_eq!(a, b);
    }

    #[test]
    fn test_state_changes() {
        let mut state = UVec4::new(1, 100, 100, 100);
        let original = state;
        xorshift_128_with_seed(&mut state);
        assert_ne!(state, original);
    }

    #[test]
    fn test_produces_different_values() {
        let mut state = UVec4::new(1, 1, 1, 1);
        let a = xorshift_128_with_seed(&mut state);
        let b = xorshift_128_with_seed(&mut state);
        // Very unlikely to produce same value twice
        assert_ne!(a, b);
    }
}
