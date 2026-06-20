---
name: bevy-shader-review
description: |
  CRITICAL: Use before and after modifying any WGSL shader. Triggers on:
  shader, wgsl, compute shader, pipeline, bind group, buffer layout,
  uniform, storage buffer, workgroup, dispatch, atomic,
  "改 shader", "shader 编译", "WGSL 错误", "buffer 对齐",
  "binding 不匹配", "storage 访问", "uniform 布局",
  shader review, shader check, shader validation
---

# Bevy Shader Review

> **Version:** 0.1.0 | **Last Updated:** 2026-06-21

你是 Bevy WGSL shader 的修改前检查 + 修改后验证工具。对每个 shader 改动执行以下流程。

## 修改前检查

### 1. Buffer Layout 对齐

确认 WGSL `@group(N) @binding(M)` 与 Rust 端 `BindGroupLayoutEntries` 一致:

```bash
# 查 Rust 端 binding 声明
search pattern="@binding|BindGroupLayoutEntries" crates/atom_terrain/src/compute/
# 查 WGSL 端 binding 声明
search pattern="@group|@binding" assets/shaders/
```

对照表:
| WGSL 声明 | Rust 端类型 | 备注 |
|-----------|------------|------|
| `var<uniform>` | `uniform_buffer::<T>(false)` | 只读 |
| `var<storage, read>` | `storage_buffer::<Vec<T>>(true)` | 只读 storage |
| `var<storage, read_write>` | `storage_buffer::<Vec<T>>(false)` | 可写 storage |
| `array<atomic<u32>>` | `storage_buffer::<Vec<u32>>(false)` | 原子计数 |

### 2. Uniform 对齐

WGSL `struct` 字段和 Rust `#[repr(C)]` struct 的 sizeof/align 必须一致:

- `vec3<f32>` = 12 bytes, `vec4<f32>` = 16 bytes
- struct 总大小必须是 16 的倍数
- 补 `pad: u32` / `pad: vec2<u32>` 保证对齐
- Rust 端用 `bytemuck::bytes_of(&uniform)` 写入，必须 `#[derive(Pod, Zeroable)]`

### 3. 跨 shader 一致性

若多个 shader 共享 binding layout，确保 struct 定义一致:

```bash
search pattern="struct GlobalUniforms" assets/shaders/
```

### 4. WGSL 语法陷阱

| 陷阱 | 修复 |
|------|------|
| `var<storage>` 无访问修饰 | 默认 `read`，需写则显式 `read_write` |
| `atomicAdd(&counters[0], 1u)` | counters 必须声明 `array<atomic<u32>>` |
| `floor()` 返回 `f32` | `u32(floor(x))` 显式转换 |
| `clamp(x, a, b)` 要求同类型 | `clamp(f32(val), 0.0, max_val)` |

## 修改后验证

### 1. 编译检查

```bash
cargo check -p atom_terrain 2>&1 | grep -E "error|warning.*shader"
```

编译通过 ≠ shader 正确，但编译**不**通过 = 立刻有问题。

### 2. 运行验证

```bash
# 必须 --release（debug 极慢）
cargo run -p atom_terrain --example chunk_loader --release 2>&1 | grep -E "error|WGSL|panic|validation"
timeout 10
```

关键检查:
- `no definition in scope for identifier` → 变量未定义
- `pipeline cache: failed to process shader` → WGSL 编译错误
- GPU validation error → buffer 大小/对齐不匹配

### 3. 视觉验证

运行 example，检查:
- 地形是否渲染（非黑屏）
- 有无闪烁/缺面/法线反转
- `Global DC: mesh sent` 日志出现

### 4. 性能检查

Shader 改动后，检查 dispatch 参数:
- workgroup size 不变跨平台: 用 `@workgroup_size(8,8,8)`（不要 `64`）
- 总线程数 ≥ grid 大小（有 guard `if gid >= gs { return; }`）

## 输出格式

完成后返回:
```
**文件**: <shader 路径>
**改动**: <一行描述>
**编译**: 通过/不通过 (<错误>)
**运行**: 通过/不通过 (<现象>)
**视觉**: 正常/异常 (<描述>)
```
