/// TODO 之后做的几个优化点。
/// 1. 排除掉完全空的区域，不进行计算。可以大幅减少gpu read back的时间消耗。
/// 2. 将main mesh compute的四个dispatch，合并到两个，也许可以减少一些时间消耗。(这个需要先把整个流程做完，避免负优化)
/// 3. gpu seam 的话，可以试试合并三个轴向的mesh到一个，前提是第一个优化点做了，之后看情况试一下。(gpu seam read back的时间消耗很大，不使用Gpu)
/// 4. 将readback的内容先拷贝到另外的buffer中，之后再回读，减少read back的buffer的尺寸。也许可以降低开销。
///     0. 这需要indirect buffer copy 去支持特定尺寸的buffer copy。目前wgpu不支持。。。。
///     1. 创建两个同样大小的缓冲区，分别记录下顶点和索引数量以及对应的内容。
///     2. 在每个chunk dispatch的最后拷贝。通过indirect dispatch, 设置线程组的数量，然后进行每线程1个数据拷贝。
///     3. 最后读取缓冲区，获取开始的顶点数量，然后根据索引数量，从缓冲区中读取数据。

#[cfg(feature = "cpu_seam")]
pub mod cpu_dc;
pub mod gpu_dc;
