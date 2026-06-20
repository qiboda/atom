# qef.spec — QEF 求解子程序

## 范围

`qef_solve.wgsl` / `main_mesh_compute_vertices.wgsl` / `atom_pqef`:
- QEF 正规矩阵构建（概率正则化）
- 3×3 线性求解（Cramer's rule 或 direct inversion）
- 安全 fallback（centroid）

## 行为契约

### 概率正则化 (Trettner & Kobbelt 2020)

**目标**: 经典 QEF `A = Σ n_i n_i^T` 在 height field 下 rank ≤ 2，奇异。加各向同性高斯不确定性 `n ~ 𝒩(μ_n, σ_n²I)` 使 A 恒满秩。

**公式**:
```
σ_n = 0.1 · voxel_size
σ² = σ_n²

A = Σ (n_i n_i^T)  +  ncross · σ² · I
b = Σ (n_i · d_i)   +  σ² · Σ p_i
```

**精度**:
- 约束方向（法线有效分量）: QEF 解保持在经典解的 1% 内
- 无约束方向（法线零分量）: 正则化推向 centroid，偏差 ≤ 0.1·voxel_size

**σ_n 选取**: `σ_n = 0.1 · voxel_size` 满足:
- `det(A) ≥ ncross³ · σ⁴ > 1e-5` 对于 `ncross ≥ 2`
- 对 QEF 解的偏移 < 1%（约束方向）
- 对平坦区域 (nz=0) 提供足够 z 方向刚度

### 求解器

**3×3 Cramer's rule**:
- `d = det(A)`
- 若 `|d| < 1e-5` → 返回 `vec3(0)`（触发 centroid fallback）
- 否则 `x_i = det(A_i) / d`，其中 `A_i` 是 A 的第 i 列替换为 b

**centroid fallback**:
- `vp = Σ p_i / ncross`
- 触发条件: `length(vp) < 1e-4`（Cramer 返回零）或 `|vp - centroid| > 2·voxel_size`（离群）

**注**: 概率正则化后，`|d| < 1e-5` 仅在 `ncross = 1` 时触发（单 crossing 无有效 QEF 解，centroid 合理）。

## 实现对照

| 位置 | 公式 | 状态 |
|------|------|------|
| `atom_pqef/src/quadric.rs` | `probabilistic_plane_quadric()` + `minimizer()` direct inversion | ✅ 正确 |
| `assets/shaders/quadric/quadric.wgsl` | 同上 WGSL 移植 | ✅ 正确 |
| `assets/shaders/terrain/compute/qef_solve.wgsl` | inline: 经典累积 + 后正则化 + Cramer's rule | ✅ 修复后正确 |
| `assets/shaders/terrain/compute/main_mesh_compute_vertices.wgsl` | 同上 | ✅ 修复后正确 |

**分歧 (已知)**: terrain shader 使用 inline 实现（手动累积 + Cramer's rule），未 `#import quadric` 模块。原因是:
- 手动累积每边在 GPU 上更高效（避免 Quadric 结构体构造/加法开销）
- Cramer's rule 与 direct inversion 等价，但直接复用结果
- 后续统一方向: 将 inline 代码移入 `quadric.wgsl` 的 `accumulate_plane()` 函数

## 验证

- 平坦 terrain (nz ≡ 0): `det(A) > 1e-5`，QEF 在 xy 方向精确，z 方向 < 0.05·voxel 偏离
- 单 crossing: centroid fallback 触发，顶点 = 交叉点（合理）
- 3-6 crossings (典型): QEF 求解，无 fallback

## 参考

- Trettner & Kobbelt. "Probabilistic Quadrics." Eurographics 2020. §3-4.
- Ju et al. "Dual Contouring of Hermite Data." SIGGRAPH 2002. §3.2 (经典 QEF)
- Garland & Heckbert. "Surface Simplification Using Quadric Error Metrics." SIGGRAPH 1997. (QEM 起源)
