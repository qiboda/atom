#[allow(dead_code)]
pub const U32_MAX: u32 = 0xFFFFFFFF;
#[allow(dead_code)]
pub const U32_MIN: u32 = 0x00000000;
#[allow(dead_code)]
pub const I32_MAX: i32 = 0x7FFFFFFF;
#[allow(dead_code)]
pub const I32_MIN: i32 = -0x80000000;

// FIXME: 超出精度，是否需要修改？因为shader中使用了这个值。
#[allow(dead_code, clippy::excessive_precision)]
pub const FLOAT_MAX: f32 = 3.402823466e+38;
#[allow(dead_code, clippy::excessive_precision)]
pub const FLOAT_MIN: f32 = 1.175494351e-38;
