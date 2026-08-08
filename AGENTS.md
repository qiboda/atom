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
| `.opencode/skills/` | **Agent 技能**（atom-workflow / issue-workflow / test / rustdoc / docs / reflect / worktree / bevy-api-lookup / bevy-shader-review / opencode-maintainer） |
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
- **问题处理闭环（强制）**：执行中遇到**任何**异常（工具失败、命令报错、编译错误、测试失败、hook 拒绝、数据不一致、流程障碍）时，**禁止静默绕过或静默降级**——包括"改用替代工具""忽略错误继续""跳过步骤""换个说法糊弄过去"。必须依次完成：
  1. **感知**：停下，识别这是问题，不把绕行当解决
  2. **诊断**：用客观证据定位根因（日志、环境变量、复现实验、对比验证），不猜
  3. **处理**：修复根因；仅在确认根因无法修复时允许 fallback，且必须在记录中说明
  4. **记录**：根因与排查路径沉淀到 `.opencode/kb/TENSIONS.md`，使其可复用
  - 绕行本身就是违规，无论结果多顺利。本规则覆盖编译错误、测试失败、hook 拒绝等一切异常。
- **agent 可自行完善项目书**：发现重复摩擦或可预防的失误时，agent 有权在 AGENTS.md / `.opencode/kb/` 中添加或修订规则以改善自身行为——规则变更随当次 commit 提交并在 commit message 中说明理由。
- **测试先行**：feature/bugfix 变更优先从能复现问题的失败测试开始（RED），再做让它通过的修复（GREEN）。先写修复再写失败测试是反模式。

## 变更前 SELF-CHECK（每次代码编辑前问自己）

0. **"这项工作有 GitHub issue 吗？"** — feature/bugfix 工作没有就先用 `/issue-workflow` 创建。
1. **"我先写了失败测试吗？"** — 没有就先用 `/test` skill 写测试再实现。
2. **"我更新了相关 kb/ 文件吗？"** — 没有就确定文件并更新（用 `/docs` skill）。
3. **"公共 API 有 `///` 文档吗？"** — 新增 pub 项时先验证 `#[deny(missing_docs)]` 合规（用 `/rustdoc` skill）。
4. **"当前工作在正确的分支/worktree 上吗？"** — 存在活跃 worktree 时（`git worktree list`），实现工作必须在 worktree 内进行；main 只允许 docs/lint/typo/反思类提交直推。不确认分支归属就不开始。
5. **"发现摩擦/不一致了吗？"** — 有就先记 `.opencode/kb/TENSIONS.md`，再处理。不跳过信号采集直接修复。

## Worktree 纪律

PR/功能分支开发使用 git worktree，位于 `.worktrees/<name>/`（gitignored），每个 worktree 对应一个功能分支，合并后清理。

- **创建时机**：多模块 feature/epic 类工作（2+ 模块、将产出设计/计划文档）时创建并切换；单文件修复/纯文档不需要。
- **流程**：`git worktree add -b feat/<name> .worktrees/<name> main` → 在 worktree 内完成实现/验证 → 合并后清理（`git worktree remove` + `git branch -D`）。
- **强制规则**：worktree 一旦创建，后续实现工作必须在 worktree 内完成；main 上不再继续实现。main 只允许 docs/lint/typo/反思类提交直推。存在活跃 worktree 时实现类提交落在 main 即流程违规，在 TENSIONS.md 记录。
- 详细命令见 `.opencode/skills/worktree/SKILL.md`。

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

## 构建检查流程

### 门禁顺序

```
cargo check --workspace    # 先确保编译通过
cargo clippy --workspace   # 再检查代码质量
cargo test --workspace     # 最后跑测试
```

### 何时运行

- 任何 Rust 代码变更后 → `cargo check --workspace`
- 触及公共 API 或新增文件 → 上述全流程
- 修改 Shader（`.wgsl`）→ 提示用户实际运行验证（编译通过 ≠ 渲染正确）

### 失败处理

- Check 失败 → 读错误，修复，重试
- 连续 ≥3 轮编译失败 → 回退 design 重审方案
- Clippy 警告 → 按项目 clippy 配置处理；`unwrap_used` 必须修复
- 测试失败 → 分析根因，修复逻辑。绝不压制测试来让代码通过

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
