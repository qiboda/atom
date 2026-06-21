# 32³ per-chunk 开放世界地形重构

## 目标
从单个 global 50³ grid 改为 64m 半径的 per-chunk 系统。

## 参数

| 项 | 值 |
|---|---|
| chunk grid | 33³ voxels（32³ + 1 ghost border） |
| voxel size | 0.5m |
| chunk 世界尺寸 | 16m³ |
| 水平视野 | 64m 半径（9×9 网格，圆形裁剪 → ~60 chunk） |
| 垂直范围 | -32m 到 32m（64m，4 层） |
| 总计 chunk | ~240（圆形裁剪 × 4 层），实际 dispatch 仅活跃的 |

## 活跃 chunk 检测

每个 chunk 的 8 角点采样密度。符号全相同 → 跳过。符号混合 → dispatch 5-pass compute。
平坦地形时活跃数 < ¼，山区 ~½。

## 改动

### 新增 `chunk_manager.rs`

- `ChunkManager` resource
- 保存 ChunkId → ChunkState 映射（空闲/计算中/就绪/已卸载）
- 每帧：根据玩家位置计算新 chunk 列表 → 对比当前 → 生成加载/卸载队列
- 加载：标记为目标 → 下一帧 dispatch compute
- 卸载：释放 GPU buffer 槽位

### 修改 `global_compute.rs` → `per_chunk_compute.rs`

- compute 系统不再有全局状态机（pass 0→1→2→...），改为 per-chunk 状态机
- 每帧遍历待计算的 chunks，逐步推进各自的 pass
- 最多 5-pass 并发的 chunk 数由 GPU 提交缓冲大小决定
- pass 0: 密度场填充（chunk 位置作为 grid_min）
- pass 1-3: edge detect + QEF + index
- pass 4: indirect draw command

### 修改 `global_pool.rs` → `shared_pool.rs`

- 从单一大 buffer 改成一个共享 pool + per-chunk slot 分配
- chunk 完成 compute 后释放 slot 给空闲列表
- 顶点/索引 buffer 可以复用同一个大 buffer（free list 管理）

### 修改 `mesh.rs`

- 从收单个 global mesh 改为收 per-chunk mesh
- readback 可选：只有碰撞需要的 chunk 才读

### 修改 `render/indirect.rs`

- 从 draw 一个 indirect command 改为 draw 多个
- 每 chunk 一条 `draw_indexed_indirect`，或 batch 到一次 draw

### shader 文件

- 基本不动（已参数化 `grid_min`/`grid_size`）
- 每 chunk dispatch 时写入对应的 uniform + buffer 偏移

## 非目标

- LOD（不做）
- chunk seam 处理（ghost voxel 方案不额外做 seam fix）
- 持久化存档
- 删除 readback 路径

## 验证方式

- `cargo check` + `cargo clippy` 零新增警告
- `top_down_game` 启动后可见 64m 范围地形
- WASD 移动时边缘 chunk 加载，远处 chunk 卸载
- 玩家在垂直 -32~32 范围内都能看到地形
