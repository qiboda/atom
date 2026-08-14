---
name: review
description: 代码审查协议：审查维度、项目约定核对清单与报告格式。subagent_review 子代理自动加载；手写审查任务时也请先加载。
whenToUse: 执行任何代码审查时（subagent_review 启动的审查子代理、或主会话中的审查任务）。
---

# 代码审查协议（Atom Terrain Engine）

审查 = 找出真实问题并给出可执行建议，不修改代码。以下协议与项目约定核对清单，
由 `subagent_review` 子代理（persona 见 `~/.dsh/cordis.patch.yml`）加载执行。

## 硬边界

- **只读**：禁止修改/创建文件、git 提交或 checkout、格式化、代码生成、下载依赖、后台任务。
- **Shell 只读**：仅 git diff/status/log/show、rg、ls、sed -n 等查看类命令。
- **编译分级**（subagent-compile 约定）：仅允许 `cargo check -p <crate>` / `cargo check --tests -p <crate>`；
  禁止 `cargo build/test/clippy/nextest/run/llvm-cov` 等重型命令（共享 `target/` 锁竞争，拖慢并行）。
- **证据原则**：每条结论必须来自真实读取的代码，标注 `file:line`；无法确认的标注「待确认」，
  禁止凭印象下结论。任务信息不足时在报告开头列出所需输入，不自行扩大范围。

## 审查维度

1. **正确性**：逻辑错误、边界条件、错误处理、资源泄漏、并发/借用问题、空指针/越界、panic 路径。
2. **项目约定核对**（对照 AGENTS.md，逐项确认）：
   - 错误处理：禁止 `unwrap()` → `expect("原因")`；不使用 thiserror/anyhow
   - 公共 API：`#[deny(missing_docs)]` + `///` 文档（RUSTDOCFLAGS="-Dwarnings" 门禁）
   - clippy 零警告（`cargo clippy --workspace -- -A dead_code -D warnings`）；rustfmt（edition 2024）
   - 测试先行 RED→GREEN；覆盖率门槛 80%（当前基线 27.54%，CI 红属预期需补测试）
   - commit 引用：`ref #N`（OPEN issue，commit-msg hook 强制）；不用 fixes/closes
   - kb 映射表：本次变更类型 → 应更新的 `.dsh/kb/` 文件是否已更新
   - hot path 零分配；GPU buffer 用 encase/bytemuck
   - 依赖克制：stdlib → workspace → Bevy 生态 → 自实现（<1 周），新依赖需四关
   - worktree 纪律：实现工作是否在 `.worktrees/` 内、分支同步是否 rebase（禁 merge）
   - Shader 变更是否必须 `--release` 实跑冒烟（`cargo run -p atom_terrain --example chunk_loader --release`，超时 30s）
   - Bevy API 不确定处：先查 `.dsh/kb/bevy/migration-index.md`，再读 `/data/codes/Bevy` 源码
3. **设计/API**：公共 API 签名与文档、模块边界、数据流清晰度、过度/欠设计、架构不变量
   （`.dsh/kb/ARCHITECTURE.md` 的 ADR）是否被破坏。
4. **性能与安全**：热路径分配、GPU buffer 处理、未定义行为、输入校验、数据表/Asset 加载路径。
5. **测试**：关键路径覆盖、边界与对抗性场景、断言是否真正验证行为（不只测 happy path）。

## 报告格式（Markdown）

1. **摘要**：审查范围、总体评价、明确结论（通过 / 修改后通过 / 不通过）。
2. **发现清单**，按严重度排序，每项含 `file:line`、问题描述、为什么是问题、修复建议：
   - `P0 阻断`：必须修复才能合并
   - `P1 主要`：应修复，影响正确性/健壮性/约定
   - `P2 次要`：建议改进
   - `P3 建议`：风格、命名、文档
3. **正面评价**：值得保留的做法。
4. **结论与后续建议**。

## 输出

- 使用与任务相同的语言回复（默认中文）。
- 只输出审查报告；不输出修改后的代码（修复建议中的短代码片段除外）。
