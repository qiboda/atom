//! 与 GPU shader 对应的 CPU 参考实现（用于对照测试与验证 shader 算法正确性）。

/// 数值边界常量（与 WGSL 内置常量对齐）。
pub mod numeric;
/// OpenSimplex 2D 噪声的 CPU 参考实现。
pub mod open_simplex;
/// 以固定种子生成置换表并驱动 OpenSimplex 噪声的封装。
pub mod open_simplex_seed;
/// xorshift128 伪随机数生成器的 CPU 参考实现。
pub mod xorshift_128;
