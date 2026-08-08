/// `u32` 最大值（`0xFFFFFFFF`），与 shader 中的 `4294967295u` 对齐。
#[allow(dead_code)]
pub const U32_MAX: u32 = 0xFFFFFFFF;
/// `u32` 最小值（`0x00000000`），与 shader 中的 `0u` 对齐。
#[allow(dead_code)]
pub const U32_MIN: u32 = 0x00000000;
/// `i32` 最大值（`0x7FFFFFFF`），与 shader 中的 `2147483647i` 对齐。
#[allow(dead_code)]
pub const I32_MAX: i32 = 0x7FFFFFFF;
/// `i32` 最小值（`-0x80000000`），与 shader 中的 `-2147483648i` 对齐。
#[allow(dead_code)]
pub const I32_MIN: i32 = -0x80000000;

// FIXME: 超出精度，是否需要修改？因为shader中使用了这个值。
/// `f32` 最大值，与 shader 中的 `3.402823466e+38` 对齐。
#[allow(dead_code, clippy::excessive_precision)]
pub const FLOAT_MAX: f32 = 3.402823466e+38;
/// `f32` 最小值（最小正正规数），与 shader 中的 `1.175494351e-38` 对齐。
#[allow(dead_code, clippy::excessive_precision)]
pub const FLOAT_MIN: f32 = 1.175494351e-38;
