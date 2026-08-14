---
name: bevy
description: |
  Bevy 0.19 技术查询与验证工具。覆盖 API 检索（不确定的 API/类型/迁移问题）和 WGSL shader 审查（改前检查 + 改后验证）。触发词：
  bevy API, how to use, API changed, migration, deprecated, removed, renamed,
  bevy 0.19, compute pipeline, bind group, render world, extract resource,
  "这个 API 怎么用", "迁移", "API 变了", "类型不对", "编译错误",
  shader, wgsl, compute shader, buffer layout, uniform, storage buffer,
  workgroup, dispatch, atomic, "改 shader", "WGSL 错误", "buffer 对齐",
  "binding 不匹配", shader review, shader validation
---

# Bevy 技术查询与验证

本项目基于 Bevy 0.19。遇到 Bevy API 问题或需要修改 WGSL shader 时，按本 skill 流程处理。

---

## 第一部分：API 检索

遇到不确定的 Bevy API 时，按以下顺序查找，找到即停。

### 1. 查 migration-index（3 秒）

```bash
grep -i "<关键词>" .dsh/kb/bevy/migration-index.md
```

`migration-index.md` 是按主题组织的 0.18→0.19 变更速查表。命中 → 读取对应行拿到答案。

### 2. 查 patterns（10 秒）

```bash
grep -i "<关键词>" .dsh/kb/bevy/0-19/patterns.md
```

`patterns.md` 是已验证的可编译代码模式。命中 → 直接复用代码片段。

### 3. 查 Bevy 源码（30 秒）

用 `Grep` 工具在 `/data/codes/Bevy/crates/` 中搜索类型定义/使用位置。

> ⚠️ **源码一致性**：编译用的 Bevy 来自 `qiboda/bevy` 的 `atom-patches` 分支
> （`[patch.crates-io]` git 引用，见根 Cargo.toml）。本地 `/data/codes/Bevy`
> 必须保持在该分支上（`git -C /data/codes/Bevy branch --show-current` 应为
> `atom-patches`）且未提交的本地修改已 push——否则查到的源码 ≠ 编译的源码。
> 修改本地 Bevy 后必须 commit + push 到 `atom-patches`，再以 git 分支状态为准。

```bash
# 找 trait 定义
grep -rn "pub trait $TRAIT" /data/codes/Bevy/crates/

# 找方法签名
grep -rn "fn $METHOD" /data/codes/Bevy/crates/
```

### 4. 查 Bevy 示例（20 秒）

```bash
find /data/codes/Bevy/examples -name "*.rs" | xargs grep -l "$KEYWORD"
```

### 5. 查 release-notes

```bash
grep -i "<关键词>" .dsh/kb/bevy/0-19/release-notes.md
```

### 常见问题速查

| 问题 | 答案 |
|------|------|
| Render Graph API | 已移除，改用 Systems + `Render` schedule |
| Bind Group 创建 | 新 API: `BindGroupLayoutEntries::sequential()` + `render_device.create_bind_group()` |
| ExtractResource | 用 `ExtractResourcePlugin::<T>::default()` 或在 `RenderApp` 手动 `insert_resource` |
| Material 定义 | `impl Material for T`, `AsBindGroup` derive, `MaterialPlugin::<T>` |
| Mesh 创建 | `Mesh::new(PrimitiveTopology, RenderAssetUsages::default())` |
| 系统调度 | `app.add_systems(Update, system)` — 不再用 `add_system` |
| Render World 通信 | `crossbeam::channel` + `Render` schedule system |

### API 输出格式

```
**API**: <类型/函数名>
**来源**: <文件:行号>
**用法**: <代码片段>
**注意**: <迁移陷阱/与旧版差异>
```

---

## 第二部分：Shader 审查

对每个 shader 改动执行以下流程。

### 修改前检查

#### 1. Buffer Layout 对齐

确认 WGSL `@group(N) @binding(M)` 与 Rust 端 `BindGroupLayoutEntries` 一致:

```bash
# 查 Rust 端 binding 声明
grep -rn "@binding\|BindGroupLayoutEntries" crates/atom_terrain/src/compute/
# 查 WGSL 端 binding 声明
grep -rn "@group\|@binding" assets/shaders/
```

对照表:
| WGSL 声明 | Rust 端类型 | 备注 |
|-----------|------------|------|
| `var<uniform>` | `uniform_buffer::<T>(false)` | 只读 |
| `var<storage, read>` | `storage_buffer::<Vec<T>>(true)` | 只读 storage |
| `var<storage, read_write>` | `storage_buffer::<Vec<T>>(false)` | 可写 storage |
| `array<atomic<u32>>` | `storage_buffer::<Vec<u32>>(false)` | 原子计数 |

#### 2. Uniform 对齐

WGSL `struct` 字段和 Rust `#[repr(C)]` struct 的 sizeof/align 必须一致:

- `vec3<f32>` = 12 bytes, `vec4<f32>` = 16 bytes
- struct 总大小必须是 16 的倍数
- 补 `pad: u32` / `pad: vec2<u32>` 保证对齐
- Rust 端用 `bytemuck::bytes_of(&uniform)` 写入，必须 `#[derive(Pod, Zeroable)]`

#### 3. 跨 shader 一致性

若多个 shader 共享 binding layout，确保 struct 定义一致:

```bash
grep -rn "struct GlobalUniforms" assets/shaders/
```

#### 4. WGSL 语法陷阱

| 陷阱 | 修复 |
|------|------|
| `var<storage>` 无访问修饰 | 默认 `read`，需写则显式 `read_write` |
| `atomicAdd(&counters[0], 1u)` | counters 必须声明 `array<atomic<u32>>` |
| `floor()` 返回 `f32` | `u32(floor(x))` 显式转换 |
| `clamp(x, a, b)` 要求同类型 | `clamp(f32(val), 0.0, max_val)` |

### 修改后验证

#### 1. 编译检查

```bash
cargo check -p atom_terrain 2>&1 | grep -E "error|warning.*shader"
```

编译通过 ≠ shader 正确，但编译**不**通过 = 立刻有问题。

#### 2. 运行验证

```bash
# 必须 --release（debug 极慢）
cargo run -p atom_terrain --example chunk_loader --release 2>&1 | grep -E "error|WGSL|panic|validation"
timeout 10
```

关键检查:
- `no definition in scope for identifier` → 变量未定义
- `pipeline cache: failed to process shader` → WGSL 编译错误
- GPU validation error → buffer 大小/对齐不匹配

#### 3. 视觉验证

运行 example，检查:
- 地形是否渲染（非黑屏）
- 有无闪烁/缺面/法线反转
- `Global DC: mesh sent` 日志出现

#### 4. 性能检查

Shader 改动后，检查 dispatch 参数:
- workgroup size 不变跨平台: 用 `@workgroup_size(8,8,8)`（不要 `64`）
- 总线程数 ≥ grid 大小（有 guard `if gid >= gs { return; }`）

### Shader 输出格式

```
**文件**: <shader 路径>
**改动**: <一行描述>
**编译**: 通过/不通过 (<错误>)
**运行**: 通过/不通过 (<现象>)
**视觉**: 正常/异常 (<描述>)
```
