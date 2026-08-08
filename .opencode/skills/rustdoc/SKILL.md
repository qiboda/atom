---
name: rustdoc
description: 检查 #[deny(missing_docs)] 合规性并识别 atom_terrain 中 pub 项缺失的 /// 文档注释。仅识别——不自动生成文档。触发词：missing_docs、rustdoc、文档注释、pub 项、cargo doc。
---

# Rustdoc — 公共 API 文档合规 Agent

## 角色

验证 `atom_terrain` 中的每个**公共项**是否都有 `///` 文档注释，强制执行 `#[deny(missing_docs)]` 合规。按文件和行号识别缺失的文档。报告发现——**绝不自动生成文档注释**。

## 触发条件

- `/rustdoc` 斜杠命令（用户发起）
- AGENTS.md 变更前 SELF-CHECK 第 3 步（新增 pub 项时）

## 工作流

### 第 1 步：运行 `cargo doc`

```sh
cargo doc --no-deps -p atom_terrain 2>&1
```

项目 pre-commit/pre-push hook 使用 `RUSTDOCFLAGS="-Dwarnings" cargo doc --no-deps -p atom_terrain`——`#[deny(missing_docs)]` 下缺失文档即编译错误。

### 第 2 步：解析警告

解析 `cargo doc` 输出中的 `missing_docs` 错误/警告。每条包含：

```
error: missing documentation for a <item type>
  --> <file>:<line>:<col>
   |
<line> | <code context>
   |
```

需要文档的项：
- `pub fn`、`pub struct`、`pub enum`、`pub trait`、`pub type`、`pub mod`
- `pub enum` 变体（每个都必须有文档）
- `pub const`、`pub static`
- `pub` trait 方法和关联类型

### 第 3 步：报告发现

以表格形式格式化输出：

```
## Rustdoc 合规检查

### 缺失文档
| 文件 | 行号 | 项 | 类型 |
|---|---|---|---|
| crates/atom_terrain/src/noise.rs | 42 | height_at | pub fn |
| crates/atom_terrain/src/lib.rs | 15 | TerrainChunk | pub struct |

### 警告计数
- 总警告数：N
- 缺失文档：M

### 结论
<CLEAN | N 项需要文档>
```

## 边界情况

| 场景 | 行为 |
|---|---|
| 提交中没有 pub API 变更 | 报告"未检测到 pub API 变更——跳过 rustdoc 检查" |
| `#[deny(missing_docs)]` 未设置 | 报告"missing_docs lint 未激活——将 `#[deny(missing_docs)]` 添加到 lib.rs"并停止 |
| `cargo doc` 因非文档错误失败 | 将编译错误与文档警告分开报告 |
| `cargo doc` 运行但无警告 | 报告 CLEAN |
| `cargo doc` 超时 | 使用 `--no-deps -j 1` 运行并重试一次 |

## 禁止事项

- **自动生成 `///` 文档注释**——仅识别缺失项；主 agent 负责编写
- **修改任何 Rust 源文件**——只读操作
- **跳过非文档错误**——即使与文档无关也要报告编译错误
- **添加 `#[allow(missing_docs)]`**——绝不抑制该 lint

## 参考

- `AGENTS.md` 编码规范 § 公共 API 强制 `#[deny(missing_docs)]`
- 架构背景见 `.opencode/kb/ARCHITECTURE.md`（文档注释中的设计理由引用）
