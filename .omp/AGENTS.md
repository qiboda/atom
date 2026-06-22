# Atom Terrain Engine

基于 Bevy 0.19 的体素平滑地形（GPU Dual Contouring + QEF）。
Bevy API 变更频繁，遇到不确定的 API 先查 `.omp/kb/bevy/migration-index.md`，没有再读 `/data/codes/Bevy` 源码。
架构导航: 高层见 `.omp/ARCHITECTURE.md`（架构不变量 + ADR），符号级见 `cargo doc --open`。

## 文档索引

| 位置 | 内容 |
| `.omp/ARCHITECTURE.md` | 架构不变量（数据流/管线/约束）+ ADR |
| `.omp/SOUL.md` | Agent 行为规范、依赖规则、测试策略 |
| `.omp/TENSIONS.md` | 摩擦日志（发现不一致时记录，不立即解决） |
| `.omp/kb/` | **知识库**（Bevy 生态 + Pi 框架 + 项目知识） |
| `.omp/skills/` | **Agent 技能** |
| `.omp/CAPABILITY-MAP.md` | 当前能力边界 |
| `agent/` | **Agent sidecar** — TypeScript BRP 客户端 |
| `.omp/rules/build-check.mdc` | 构建检查流程 |

## Workspace 当前状态

目前 workspace **包含 `crates/atom_terrain` + `agent/` (TypeScript sidecar)**。其他 crate（atom_render, atom_shader_lib,
atom_ability, atom_layertag, atom_datatables, atom_core, atom_math, atom_renderdoc,
atom_cel_shader, atom_pqef, atom_utils）暂时移出，后续逐步迁入 Bevy 0.19。

**重要**: Bevy debug 构建极慢（~19s 启动，30s+ 出首帧）。运行/测试必须用 `--release`。
地形验证: `cargo run -p atom_terrain --example chunk_loader --release`（超时 30s）。
Agent 验证: `cargo run -p atom_terrain --example top_down_game --release`（需 npm deps: `cd agent && npm install`）。
直接跑二进制需先 `ln -sf $(pwd)/assets target/release/examples/assets`（Bevy 从 exe 目录找 assets）。

## 项目配置优于全局
所有扩展和配置放在项目目录下（`.omp/`），不全局安装——clone 即获得完整 agent 能力。

## 编码规范

- 错误处理: 禁止 `unwrap()` → 统一 `expect("原因")`；不使用 `thiserror`/`anyhow`
- 公共 API: 强制 `#[deny(missing_docs)]` + `///` rust-doc (RFC 1574)
- Shader: 通过 `AssetServer::load` 在 Startup system 加载，不用 `DirectAssetAccessExt`
- 格式化: `rustfmt.toml` (Unix 换行, edition 2024)；clippy 零警告
- 构建检查流程见 `rules/build-check.mdc`
- 代码模式以 `crates/atom_terrain/src/` 实际代码为准
