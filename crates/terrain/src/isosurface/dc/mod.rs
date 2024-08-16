/// TODO 之后做的几个优化点。
/// 1. 排除掉完全空的区域，不进行计算。可以大幅减少gpu read back的时间消耗。
/// 2. 合并gpu buffer，减少创建buffer的次数。可以大幅减少prepare buffer的时间消耗。
/// 3. 将main mesh compute的四个dispatch，合并到两个，也许可以减少一些时间消耗。(这个需要先把整个流程做完，避免负优化)
/// 4. gpu seam 的话，可以试试合并三个轴向的mesh到一个，前提是第一个优化点做了，之后看情况试一下。
/// 5. 将readback的内容先拷贝到另外的buffer中，之后再回读，减少read back的buffer的尺寸。也许可以降低开销

#[cfg(feature = "cpu_seam")]
pub mod cpu_dc;
pub mod gpu_dc;
