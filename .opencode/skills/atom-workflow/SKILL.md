---
name: atom-workflow
description: 强制执行 atom 项目工作流 — issue 驱动开发、文档同步、测试先行、逐步验证、提交纪律。用于本仓库的任何 feature、bugfix 或代码变更。
---

# Atom 工作流

本项目遵循严格的工作流。每次代码变更都必须执行以下规则。

---

## 🛑 触发：预实现门禁（立即执行）

**加载此 skill 的瞬间，你即进入门禁模式。**

**前置条件**：进入门禁之前，grill-me（第 0 步）必须已完成"shared understanding
reached"。如果尚未调用 `/grill-me`，请返回并先完成。

在创建任何 todos、读取任何源文件、编写任何代码之前——你必须向用户逐一确认以下检查清单。

```
🛑 预实现门禁

在继续之前，我将逐项检查门禁的每个步骤：

☐ 第 0 步 — GRILL-ME（前置条件）
   已达成 shared understanding
   → [必须确认]

☐ 第 0.5 步 — WORKTREE（grill 共识后立即判断）
   需求是否需要 worktree？（feature/epic、2+ 模块、将产出 .omo/plans/*.md
   或 .omo/designs/*.md）
   → 需要则**立即创建并切换**（/worktree）：grill 共识达成后、产出任何 .omo
     文件之前。plan/design 直接在 worktree 内创建，随实现 PR 提交。
   → 创建后写 `.worktrees/<name>/.omo/handoff.md`（用途 + issue URL + 决策），
     运行 `scripts/open-worktrees.sh <name>` 启动，剩余工作移交 worktree agent。
   → 不需要（单文件修复/纯 docs）→ 跳过，继续 main 工作区。
   → [必须展示 worktree 名称 + handoff 已写入]

☐ 第 1 步 — ISSUE
   → 按 §3 需求流程创建/管理 issues
   → [必须向用户展示 issue URL，或 epic + 子 issue 列表]

☐ 第 2 步 — PLAN（仅单文件变更可跳过）
   计划 agent 已运行且已获批准
   → [必须展示计划摘要]

☐ 第 3 步 — TESTS（RED 阶段，独立 QA）
   → 委派 `test-agent`（独立 QA，认知独立于实现）从 spec 设计并编写失败测试
   → GREEN 后由 `test-agent` 独立复验（不携带实现偏见的再验证 + spec 偏差报告）
   → 测试约定/模式见 `test` skill
   → [必须展示测试失败输出 + 复验通过]

☐ 第 4a 步 — RUSTDOC
   → `RUSTDOCFLAGS="-Dwarnings" cargo doc --no-deps -p atom_terrain` 无警告
   → [必须展示无警告输出]

☐ 第 4b 步 — DOCS（kb/）
   → 对照 AGENTS.md「kb 映射表」识别并更新 kb/ 文件
   → [必须列出文件清单]

☐ 第 4c 步 — 决策记录
   → 检查相关架构决策是否记录到 .opencode/kb/ARCHITECTURE.md 的 ADR 章节
   → 如缺失，先补充再继续
```

**在上述所有门禁步骤（1-4c）完成并向用户展示之前，
严禁使用任何 edit/write/bash 工具进行实现。**

如果你发现自己在门禁未完成时就开始编写代码，立即停止，
回到第 0 步。这是硬性阻断——feature/bugfix 工作无例外。

### 例外（可跳过门禁的情况）

门禁不适用于：
- 纯文档变更
- Lint 修复
- Typo 修复
- 为已有代码添加测试

> ⚠️ **跳过门禁并不意味着跳过实现后审查。**
> 下方"实现后审查"章节适用于所有变更，
> 包括纯文档变更。门禁和审查是两个独立流程——
> 门禁是预实现阶段，审查是实现后阶段。

### 门禁完成信号

当所有步骤完成后，明确宣布：

```
✅ 门禁完成 — 进入实现阶段
```

只有此时才能创建 todos 并开始编辑文件。

---

## 规则（按优先级排序）

### 1. 问题处理闭环（执行中任何异常，最高优先级）

执行中遇到**任何**异常（工具失败、命令报错、配置错误、数据不一致、流程
障碍、输出可疑）时，**禁止静默绕过或静默降级**——包括"改用替代工具"
"忽略错误继续""跳过步骤""换个说法糊弄过去"。必须依次完成：

1. **感知**：停下，识别这是问题，不把绕行当解决
2. **诊断**：用客观证据定位根因（日志、环境变量、复现实验、对比验证），不猜
3. **处理**：修复根因；仅在确认根因无法修复时允许 fallback，且必须在记录中说明
4. **记录**：根因与排查路径沉淀到 `.opencode/kb/TENSIONS.md`（摩擦日志），使其可复用

绕行本身就是违规，无论结果多顺利。本规则覆盖编译错误、测试失败、hook
拒绝等一切异常。遇到疑似已知问题时，先查 `.opencode/kb/TENSIONS.md`，
命中则照「排查路径」复现诊断。

### 2. 文档同步（关键）

任何影响行为、公开 API、数据结构、配置或工作流的代码变更，必须在同一次 commit 中更新相关 `.opencode/kb/` 文件和 `AGENTS.md`。

权威的「变更类型 → kb/ 文件」映射表见 `AGENTS.md` § kb 映射表——单一数据源，此处不重复。高频项：Bevy API 变更 → `kb/bevy/migration-index.md`；架构决策 → `kb/ARCHITECTURE.md`；摩擦 → `kb/TENSIONS.md`。

### 3. 需求流程（关键）

编写任何 feature 或 bugfix 代码之前，必须走完整 Issue 生命周期。

#### 3a. 模式选择

- 计划有 2+ 任务批次且各批次有独立交付物 → **Epic 模式**（epic + 子 issues）
- 否则 → **单 issue 模式**

#### 3b. 单 issue 创建

```sh
gh issue create --title "<title>" --body-file /tmp/issue-body.md --label "A-<area>,C-<category>"
```

用 `gh issue view <N>` 验证，记录 issue 编号，返回 URL。

#### 3c. Epic 模式

1. 创建 epic（父 issue）：
   ```sh
   gh issue create --title "<epic title>" --body-file /tmp/epic-body.md --label "A-<area>,C-Feature"
   ```
2. 对每个子 issue 用 `--parent` 创建：
   ```sh
   gh issue create --title "<sub-issue title>" --body-file /tmp/sub-issue-body.md --label "A-<area>,C-Feature" --parent <epic-N>
   ```
3. 子 issue body 模板：
   ```markdown
   > **Parent**: #<epic-N>
   > **Plan**: .omo/plans/<epic-name>.md
   > **Batch**: <N>
   > **Depends on**: #<sub-X>（如无则为 "—"）

   ## 描述
   <任务描述>

   ## 验收标准
   <验收标准>
   ```

#### 3d. 批次跟踪（Epic）

计划文件（`.omo/plans/<epic>.md`）是权威跟踪文档，用 `## Tasks` 表格：

```markdown
### Batch <N>
| Status | Issue | Task | Depends On |
|--------|-------|------|------------|
| pending | #<N> | <单行描述> | — |
| in_progress | #<N> | <单行描述> | #<X> |
| done | #<N> | <单行描述> | #<X>, #<Y> |
```

- `Status`：`pending` | `in_progress` | `done`；只更新状态，绝不删除行
- **批次切换**：完成当前批次全部子 issues → 状态改 `done` → 向用户报告 → 等确认再开下一批
- **新增子 issue**：实现中发现新工作 → 加任务行 + `--parent` 建 issue + 重估 DAG 依赖

#### 3e. 批次/最终关闭（PR 合并到 main 后）

```sh
gh issue close <sub-N1> <sub-N2> <sub-N3>     # 先关子 issues
gh issue comment <sub-N> --body "Fixed by #<PR-N>"
gh issue close <epic-N>                        # 再关 epic
gh issue comment <epic-N> --body "All sub-issues completed:\n- #<sub-N1>: <title>..."
```

以下情况跳过 issue 流程：重构、文档、lint 修复、typo。

提交引用：`ref #N`（feat/fix），不使用 `fixes #N` / `closes #N`（避免自动关闭）。
Epic 工作中，每个 commit 引用其子 issue（`ref #<sub-N>`）。

**这适用于所有 commit——chore、docs、scripts 均无例外。**

### 4. 计划先行

**多步工作不可妥协。** 以下情况必须运行计划流程：多步任务（2+ 模块）、架构变更、新增 crate、需求范围模糊。

计划输出到 `.omo/plans/*.md` 文件，包含任务批次排序和验证门禁。不要跳过这一步自己口头描述计划。

仅以下情况可跳过计划：真正的单文件修复、测试添加、文档更新。

### 5. 测试先行

Feature 和 bugfix 工作遵循 RED → GREEN → REFACTOR：
- 先写失败测试，确保因正确原因失败
- 然后实现
- 探索性变更可以先写代码后补测试
- 纯重构：先用特征测试锁定当前行为
- **覆盖率硬门槛**：总行覆盖率 < 80% 不达标（`just coverage` 或 CI `cargo llvm-cov nextest --workspace --release --fail-under-lines 80`）。当前基线 27.54%，CI 红属预期，先补测试再合并。AGENTS.md 为单一数据源，此处不重复细节。

### 6. 逐步验证

每次代码变更后：
- `cargo check --workspace` → 必须通过
- `lsp_diagnostics` 在变更文件上无错误
- **GPU/Shader 变更必须在提交前 `--release` 实跑冒烟**：WGSL 没有 borrow
  checker，编译通过 ≠ 渲染正确。地形验证用
  `cargo run -p atom_terrain --example chunk_loader --release`（超时 30s）。
  冒烟输出要有"渲染终态证据"（Global DC mesh sent 日志、地形非黑屏），
  不能只看 exit 0。

**失败处理**：
- Check 失败 → 读错误，修复，重试
- 连续 ≥3 轮编译失败 → 回退 design 重审方案
- Clippy 警告 → 按项目 clippy 配置处理；`unwrap_used` 必须修复
- 测试失败 → 分析根因，修复逻辑。**绝不压制测试来让代码通过**

### 7. 提交前本地验证

```sh
cargo check --workspace
cargo clippy --workspace -- -A dead_code -D warnings
cargo nextest run --workspace    # nextest 未装则 cargo test
RUSTDOCFLAGS="-Dwarnings" cargo doc --no-deps -p atom_terrain   # 新 pub 项时
```

全部通过后才可 commit 或 push。**pre-push 门禁在 push 才触发，提交阶段
零拦截**——每个实现 commit 前都必须自跑完整门禁，不能只跑 cargo check。

### 8. 禁止类型逃逸

- 生产代码中永不使用 `unwrap()` —— 使用 `.expect("原因")` 或正确的错误处理
- 永不使用 `as any`、`@ts-ignore`、`#[allow()]` 等压制类型错误

### 9. 分支策略

Feature 分支工作流：大部分工作在分支上进行，通过 PR 合并。
简单修复（typo、配置、单行变更）可直接提交到 main。

**Worktree 是分支策略的强制部分**：一旦创建 worktree（`git worktree add`），
后续实现工作必须在 worktree 内进行——main 上的实现继续即流程违规。
main 只允许纯文档（docs/lint/typo/反思）直推。

**分支同步一律 rebase，不用 merge**：worktree 分支同步 main 用
`git rebase origin/main`（绝不用 `git merge`，保持线性历史，避免合并提交）；
PR 合并用 rebase/squash。开始实现前用 `git worktree list` +
`git branch --contains HEAD` 确认所在分支；不确认分支归属就不开始。

```
main      ●──●──●──●────────●  (主干，仅 docs/lint/typo/反思)
               \          /
feat/xxx      ●──●──●──┘   (worktree 分支，通过 PR 合并)
```

### 10. 标签强制

创建 GitHub issue 或 PR 时，必须附加至少一个 **A-**（area）和一个 **C-**（category）标签；**D-**/**P-**/**S-** 可选但建议添加。

完整分类体系（前缀含义、全部标签表）见 `.opencode/kb/github/labels.md`——单一数据源，此处不重复。

`gh issue create --label "C-Bug,A-Terrain"` 或 `gh pr create --label "C-Feature,A-Compute"`。

### 11. 摩擦记录

当用户纠正 AI 行为（矛盾、范围扩张、约束遗漏、方案偏离）时——
先记 `.opencode/kb/TENSIONS.md`（摩擦日志），再处理。不跳过信号采集直接修复。

**问题排查根因**（执行中异常）：沉淀到 TENSIONS.md 的排查路径，使其可复用。

---

## 📋 可用 Skills

Atom 项目为特定工作流步骤提供以下 opencode skills：

| Skill / Agent | 斜杠命令 | 用途 | 门禁步骤 |
|---|---|---|---|
| test-agent | `task(subagent_type="test-agent")` | 独立 QA 验证者——从 spec 设计测试、独立复验实现 | 第 3 步 — TESTS |
| test | `/test` | 测试约定与模式（TDD/BDD 写法、nextest 命令） | 第 3 步 — TESTS |
| reflect | `/reflect` | 编写实现后反思（含 User corrections） | 实现后 |
| worktree | `/worktree` | 管理 git worktrees（创建/删除） | 第 0.5 步 — WORKTREE |
| bevy | `/bevy` | Bevy API 检索 + Shader 审查 | 编码规范 §Bevy API |

> Issue 生命周期（§3）与 rustdoc/docs 检查已并入本 skill——不再单独成 skill。

当门禁清单显示 `→ 调用 /<command>` 时，加载对应的 skill 并按其工作流执行。
每个 skill 的详细说明见 `.opencode/skills/<name>/SKILL.md`。

---

## 🔍 实现后审查（自动化）

实现完成后，运行自动化审查以在变更进入仓库前捕获问题。

### 第 1 步：提交

先提交实现——始终如此。不要在提交前运行审查。

```
git add <files>
git commit -m "feat: description

ref #N"
```

### 第 2 步：运行审查

对当前变更触发 `/review-work`。审查会并行运行多个 agent：
目标验证、QA 执行、代码质量、安全审计和上下文挖掘。

**Epic 工作**：两层审查——
- **每个子 issue**：每个子 issue commit 后，审查该子 issue 的变更
- **PR 前**：所有子 issue 完成后，审查完整 PR diff 以发现集成问题

### 第 3 步：处理发现的问题

针对审查报告的每个问题：

| 问题类型 | 处理方式 |
|---|---|
| 与当前工作相关，影响 ≤3 个文件 | 直接自动修复 |
| 与当前工作无关 | 创建 GitHub issue（`gh issue create`） |
| 相关但影响 >3 个文件 | 创建 GitHub issue |

### 第 4 步：重新审查（最多 2 轮）

修复问题后，重新运行审查以验证修复正确。
如果在 2 轮后审查仍然报告阻塞问题，为剩余问题
创建 issues 并在 commit message 中注明。

### 第 5 步：完成

- 所有范围内问题已解决 → 提交，等待用户 push 指令
- **用户确认 push 后、执行 push 前**：调用 `/reflect` 编写实现后反思，
  反思 commit（含 `ref #N`）与实现代码**同批 push**，随 PR 合并落在 main
- **执行检查（防止遗漏）**：push 命令发出前，确认 `.opencode/kb/project/reflections.md`
  已含本次工作的反思条目——若用户确认 push 但反思未写，先 `/reflect` 再 push。
  push 后发现遗漏时，补写反思并追加 commit（`docs: reflection ...` + `ref #N`）再 push。

### 第 6 步：Push 后关闭 issue（强制，勿忘）

**push 成功到达 `origin/main` 后**，必须完成 issue 收尾——这是流程的
一部分，不是可选项：

1. **追加完成 comment**（`gh issue comment <N>`，遵守 comments.md"永远追加"规范）：
   - 实现摘要 + 验收标准逐项状态（✅/⛔）
   - commit 列表（`git log --oneline origin/main@{1}..HEAD` 或等价范围）
   - 与 issue 原方案的偏差及原因（如方案被外部约束阻断、用户批准放弃）
2. **关闭 issue**（`gh issue close <N>`）——HARD BLOCK：只在 push 后关闭，
   push 前绝不关闭。
   - 单 issue：直接关闭
   - Epic：先关所有子 issues（每个注明 `Fixed by #<PR-N>`），再关 epic
     并在 epic 上记录总结 comment

---

## 📝 反思记录

每次 feature 或 bugfix 实现后，调用 `/reflect`（reflect skill）
编写实现后反思并追加到 `.opencode/kb/project/reflections.md`。

**时机（强制）**：用户确认 push 后、执行 push 前——反思 commit 与实现
同批 push 随 PR 合并落在 main。

完整反思工作流见 `.opencode/skills/reflect/SKILL.md`。

---

## 提交风格

- `feat:` / `fix:` / `test:` / `refactor:` / `docs:` / `chore:`
- 原子提交：每次提交一个逻辑单元
- 每个 commit 引用其 issue：`ref #N`（epic 工作引用子 issue）
- 一个 PR 可以包含多个子 issue commit（每个带有各自的 `ref #<sub-N>`）
- 仅在用户明确指令时推送（绝不自动推送）

## 代码风格

遵循 AGENTS.md「编码规范」与「Rustdoc 合规」——单一数据源，此处不重复。要点：Rust edition 2024；错误处理 `expect("原因")` 不用 `unwrap()`；公共 API 强制 `#[deny(missing_docs)]`；clippy 零警告；Bevy API 不确定先查 `.opencode/kb/bevy/migration-index.md`；Shader 通过 `AssetServer::load` 加载。
