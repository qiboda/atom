# Atom Terrain Engine

基于 Bevy 0.19 的体素平滑地形（GPU Dual Contouring + QEF）。
Bevy API 变更频繁，遇到不确定的 API 先查 `.opencode/kb/bevy/migration-index.md`，没有再读 `/data/codes/Bevy` 源码。
架构导航: 高层见 `.opencode/kb/ARCHITECTURE.md`（架构不变量 + ADR），符号级见 `cargo doc --open`。

## 文档索引

| 位置 | 内容 |
| `.opencode/kb/ARCHITECTURE.md` | 架构不变量（数据流/管线/约束）+ ADR |
| `.opencode/kb/TENSIONS.md` | 摩擦日志（发现不一致时记录，不立即解决） |
| `.opencode/kb/` | **知识库**（Bevy 生态 + 项目知识 + GitHub 约定） |
| `.opencode/kb/github/` | **GitHub 约定**（labels 标签体系 / comments 评论规范） |
| `.opencode/skills/` | **Agent 技能**（atom-workflow / test / reflect / worktree / bevy） |
| `.opencode/agent/` | **Subagent**（test-agent：独立 QA 验证者——从 spec 设计测试、独立复验实现） |
| `.githooks/` | **git hooks**（commit-msg: ref #N 强制；pre-commit: fmt/check/doc/bevy_lint；pre-push: 全门禁 + ref #N 验证） |
| `.github/` | **CI**（ci.yml：fmt/clippy/doc/nextest 门禁） |

## Issue 驱动开发（强制）

**每个 commit 必须引用 GitHub issue：`ref #N`。** 无例外——包括 chores、docs、scripts。
`ref #N` 必须指向 OPEN issue（commit-msg hook 强制校验）。epic 工作的每个 commit 引用其子 issue（`ref #<sub-N>`）。

```
feat: add thing

ref #26
```

- 使用 `ref #N`，**不使用** `fixes #N` / `closes #N`（避免自动关闭 issue）
- issue 只在 push 成功到达 `origin/main` 后手动关闭（atom-workflow 第 6 步）
- feature/bugfix 工作强制走 `atom-workflow` skill 的预实现门禁
- 完整规则见 `.opencode/skills/atom-workflow/SKILL.md`

## 品质准则

精益求精，追求完美。每一行代码、每一次提交、每一个决策，都应以最高标准衡量。

- 代码不行就重构，不要留着凑合；设计不对就推翻，不要叠加补丁
- **问题处理闭环（强制）**：执行中遇到**任何**异常，禁止静默绕过或静默降级。依次完成感知 → 诊断 → 处理 → 记录（沉淀到 `.opencode/kb/TENSIONS.md`）。完整规则见 `atom-workflow` skill §1——绕行本身就是违规。
- **agent 可自行完善项目书**：发现重复摩擦或可预防的失误时，agent 有权在 AGENTS.md / `.opencode/kb/` 中添加或修订规则以改善自身行为——规则变更随当次 commit 提交并在 commit message 中说明理由。
- **测试先行**：feature/bugfix 变更从失败测试开始（RED），再做修复（GREEN）。先写修复再写失败测试是反模式。见 `test` skill。

## 变更前 SELF-CHECK（每次代码编辑前问自己）

0. **"这项工作有 GitHub issue 吗？"** — feature/bugfix 工作没有就按 `atom-workflow` skill §3 创建。
1. **"我先写了失败测试吗？"** — 没有就委派 `test-agent`（独立 QA）从 spec 写失败测试再实现。
2. **"我更新了相关 kb/ 文件吗？"** — 没有就对照「kb 映射表」确定文件并更新。
3. **"公共 API 有 `///` 文档吗？"** — 新增 pub 项时先验证 `#[deny(missing_docs)]` 合规（见「Rustdoc 合规」）。
4. **"当前工作在正确的分支/worktree 上吗？"** — 存在活跃 worktree 时（`git worktree list`），实现工作必须在 worktree 内进行；main 只允许 docs/lint/typo/反思类提交直推。不确认分支归属就不开始。
5. **"发现摩擦/不一致了吗？"** — 有就先记 `.opencode/kb/TENSIONS.md`，再处理。不跳过信号采集直接修复。

## Worktree 纪律

PR/功能分支开发使用 git worktree，位于 `.worktrees/<name>/`（gitignored），每个 worktree 对应一个功能分支，合并后清理。

- **创建时机（强制）**：需求经 grill-me 确认是需要 worktree 的工作（feature/epic、2+ 模块、将产出 `.omo/plans/*.md` 或 `.omo/designs/*.md`）时，**grill 共识达成后立即创建并切换**；单文件修复/纯文档不需要。判断口诀：**一旦确定"这次要产出 .omo 文件"→ 先开 worktree 再写文件**（untracked 文件不会跨 checkout 迁移）。
- **主 session 移交（强制）**：创建后主 session 只做两件事——写 `.worktrees/<name>/.omo/handoff.md`（用途 + issue URL + 已锁定决策），然后运行 `scripts/open-worktrees.sh <name>` 自动启动（新终端 + setsid 脱离进程组）。剩余工作全部移交 worktree 内 agent，主 session 不再参与。
- **会话启动规则（强制）**：worktree 内 opencode 会话启动后第一步必须读取 `.omo/handoff.md` 获取上下文契约，之后才允许开始任何工作。
- **强制规则**：worktree 一旦创建，后续实现工作必须在 worktree 内完成；main 只允许 docs/lint/typo/反思类提交直推。存在活跃 worktree 时实现类提交落在 main 即流程违规，在 TENSIONS.md 记录。
- 完整流程、命令与清理（含 `--close` 终止进程 + 删 worktree）见 `.opencode/skills/worktree/SKILL.md`。

## 决策记录

架构级决策（跨子系统约束、库选型、数据流变更）必须记录到 `.opencode/kb/ARCHITECTURE.md` 的 ADR 章节——自包含记录 **what + why + why-not**。格式见该文件 ADR 模板。

## Workspace 当前状态

目前 workspace **包含 `crates/atom_terrain`**。其他 crate（atom_render, atom_shader_lib,
atom_ability, atom_layertag, atom_datatables, atom_core, atom_math, atom_renderdoc,
atom_cel_shader, atom_pqef, atom_utils）暂时移出，后续逐步迁入 Bevy 0.19。

**重要**: Bevy debug 构建极慢（~19s 启动，30s+ 出首帧）。运行/测试必须用 `--release`。
地形验证: `cargo run -p atom_terrain --example chunk_loader --release`（超时 30s）。
直接跑二进制需先 `ln -sf $(pwd)/assets target/release/examples/assets`（Bevy 从 exe 目录找 assets）。

## 编码规范

- 错误处理: 禁止 `unwrap()` → 统一 `expect("原因")`；不使用 `thiserror`/`anyhow`
- 公共 API: 强制 `#[deny(missing_docs)]` + `///` rust-doc (RFC 1574)
- Shader: 通过 `AssetServer::load` 在 Startup system 加载，不用 `DirectAssetAccessExt`
- 格式化: `rustfmt.toml` (Unix 换行, edition 2024)；clippy 零警告
- 代码模式以 `crates/atom_terrain/src/` 实际代码为准
- **构建门禁**（cargo check/clippy/test 何时跑、失败怎么处理）见 `atom-workflow` skill §6-7

### Rustdoc 合规

- 新增 pub 项后运行 `RUSTDOCFLAGS="-Dwarnings" cargo doc --no-deps -p atom_terrain`——`#[deny(missing_docs)]` 下缺失文档即编译错误
- 需文档的项：`pub fn/struct/enum/trait/type/mod`、enum 变体、`pub const/static`、trait 方法
- 只识别缺失项并报告，不自动生成 `///` 文档；绝不加 `#[allow(missing_docs)]`

### kb 映射表（变更 → 需更新的 kb/ 文件）

| 变更类型 | 需更新的文件 |
|---|---|
| Bevy API / ECS / 渲染管线变更 | `kb/bevy/migration-index.md` + `kb/bevy/0-19/patterns.md` |
| 新发现的 API 陷阱 | `kb/bevy/migration-index.md`（grep 命中则更新对应行） |
| 架构决策、数据流、ADR | `kb/ARCHITECTURE.md`（ADR 章节，what + why + why-not） |
| 问题排查、工具链摩擦 | `kb/TENSIONS.md`（格式 `- **YYYY-MM-DD**: <问题>`，只捕获信号不解决） |
| 游戏系统实现 | `kb/project/game/README.md` |
| 项目级约定 | `AGENTS.md`（索引——一句话摘要，绝不重复内容） |
| 新 skill / 插件 / workflow | `AGENTS.md`（文档索引 + skills 列表） |
| 标签约定 / 评论约定 | `kb/github/labels.md` / `kb/github/comments.md` |

**命令/术语全仓搜索（强制）**：变更涉及命令、API 名称、组件路径、crate 名等被其他文档引用的标识符时，必须全仓 grep 找全所有引用点逐一核对，不能只更新"主要"文件。

**kb 维护纪律**：不创建新 kb/ 文件（优先并入现有文件）；无代码变更上下文不修改 kb/；AGENTS.md 是索引，kb/ 是唯一数据源；不硬编码版本号。

## 工作习惯

**先读 AGENTS.md → 查 kb/ → 查 `/data/codes/Bevy` 源码 → 再动手。** 复用项目既有模式，不凭空设计。

**Shader 改后必须 `--release` 实际运行验证。** 编译通过 ≠ 渲染正确。WGSL 没有 borrow checker。

**Hot path 零分配。** GPU buffer 用 encase/bytemuck。

**依赖克制。** 能不用就不加。新引入需过四关：stdlib 有？→ workspace 有？→ Bevy 生态有？→ 自实现 < 1 周？

## Agent 能力边界

Agent 不能替代人类判断的领域：

- Shader 视觉效果（渲染质量、光照参数）
- 地形参数调优（voxel size、chunk 范围、密度场函数）
- 游戏设计决策（biome 类型、怪物、技能参数）
- 新 crate 引入决策
- GPU 性能瓶颈判断和优化方向
