# density-sampling.spec — 密度场采样子程序

## 范围

`edge_detect.wgsl` / `voxel_cross_points.wgsl`:
- `trilinear_sample(p: vec3<f32>) -> f32`
- 法线估计 `estimate_normal(p: vec3<f32>) -> vec3<f32>`
- binary search 定位交叉点

## 行为契约

### `trilinear_sample`

**目标**: 对连续密度场 `density(x,y,z) = y - height(x,z)` 在任意世界坐标 `p` 处采样，
返回三线性插值密度值，使得二分搜索能在连续 isosurface 上收敛。

**输入空间** (pre):
- `p` ∈ `[grid_min, grid_min + (grid_size+1)·voxel_size]`
- `density` buffer 已填充 `(grid_size+1)³` 个 grid 点值

**输出精度** (post):
- `|trilinear_sample(p) - 连续密度| < 0.01`，假设 grid 点密度精确、isosurface 曲率半径 > 2·voxel_size

**边界行为**:
- `p` 超出 grid → clamp 到边界 grid 点，不 extrapolate
- `p` 恰好在 grid 点上 → 返回精确密度值（权重退化）
- `p` 在 grid 点中点 → 返回 8 个角点的加权均值

**禁忌** (anti):
- 禁止 `round()` 量化 → 会导致阶梯函数
- 禁止超过 8 次 density buffer 读 → O(1) 开销
- 禁止分支发散（所有 lane 覆盖所有角点读）

### `estimate_normal`

**目标**: 用中心差分 `(density(p+h) - density(p-h)) / (2h)` 估计 isosurface 在 `p` 处的标准化梯度。

**精度**: 在平滑曲面 (`|Δ²h| < 1`) 上法线角度误差 < 5°。

**边界**: `h = voxel_size * 0.5`。`p±h` 超出 grid → clamp。

### binary search 收敛

**目标**: 在 edge `[p0, p1]` 上二分搜索 isosurface 位置，8 次迭代内收敛到 `|density(pos)| < 1e-3`。

**输入**: `sign(d0) ≠ sign(d1)`（sign change 已由端点的精确 grid 密度确定）

**算法**: 中点 `dmid = trilinear_sample(mid)`，追踪 sign 组。

## 为什么必须三线性插值

| 方案 | 二分搜索行为 | 交叉点位置 | 法线质量 |
|------|-------------|-----------|---------|
| `round()` 最近邻 | 在阶梯函数上收敛到体素边界 | 锁在体素边界 | 跳变，不连续 |
| **三线性插值** | **在连续密度场上收敛** | **isosurface 真实位置** | **平滑，连续** |

当 grid 偏移时:
- round(): 体素边界位移 → 交叉点位移 → mesh 变形
- trilinear: 交叉点连续跟踪 isosurface → mesh 不变

## 参考

- Trettner & Kobbelt. "Probabilistic Quadrics." Eurographics 2020.
- Dual Contouring 原始二分搜索: Ju et al. 2002, §3.1
