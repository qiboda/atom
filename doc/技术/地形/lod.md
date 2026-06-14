---
date created: 2024-02-01 22:19
tags:
  - clipmap
  - terrain
---


首先使用clipmap来决定周围的lod，通过lod确定octree的分割到什么级别。来决定周围Chunk的size，进行创建。

# octree lod

用于分割场景，决定chunk的大小。

# clipmap


| 起始距离 | 最终距离 | 八叉树深度 |
| ---- | ---- | ----: |
| 0    | 16   |     7 |
| 16   | 32   |     6 |
| 32   | 64   |     5 |
| 64   | 128  |     4 |
| 128  | 256  |     3 |

距离以chunk的中心点进行计算。
如下图所示
![[Clipmap 2024-02-01 00.41.40.excalidraw]]

其中距离指的是切比雪夫距离(Chebyshev distance)。
数学上，切比雪夫距离是将2个点之间的距离定义为其各坐标数值差的最大值。
$$
D_c = max(|x_1 - x_2|, |y_1 - y_2|)
$$
[[atom/技术/地/attachmen/cd47f3d47be4c615a15526def7cf3729_MD5.jpeg|切比雪夫距离]]
![[技术/地形/attachmen/cd47f3d47be4c615a15526def7cf3729_MD5.jpeg|切比雪夫距离]]


# todo

- [ ] 遮挡剔除, 基于chunk的遮挡剔除。避免生成mesh和八叉树的构建。
- [ ] 视锥剔除，摄像机范围外的chunk一律剔除。不立即构建八叉树。检测性能压力，性能较好时，再加载。
