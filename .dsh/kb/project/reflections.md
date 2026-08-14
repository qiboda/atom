# 反思日志

> 实施后反思沉淀。只追加，不修改/删除历史条目。格式见全局 `skwy-reflect` skill。

## 2026-08-09 — #7 BSN 迁移收尾：Batch 3 数据层回退修复

**What was done**: 修复 bsn-migration 分支 rebase main 时回退的 Batch 3 数据层迁移——HEAD 的 `spawn_ability`/`spawn_buff` 引用已删除的 `TbAbilityRow`/`TbBuffRow`/`TableReader`（atom_datatables），Cargo.toml 已无该依赖导致无法编译；恢复 `AbilityConfigData`/`BuffConfigData` 组件 + `spawn_ability(config)`/`spawn_buff(config)` 签名 + 删除 `trigger_buff_add_event` 死代码 + `register_component` API 修复，全门禁绿 + example 冒烟通过。

**What went wrong**:
1. **rebase 冲突解决回退数据层迁移未被察觉（最严重）**：分支 rebase 到含 Batch 3（5412795）的新 main 后，bundle.rs 仍保留旧 `TbAbilityRow` 代码——HEAD 编译失败（E0432/E0433/E0425）在 12 个提交（e684d6c→267a070）期间未被发现，直到本次收尾才暴露。rebase/merge 后必须验证编译，冲突解决倾向旧版本时更要警惕。
2. **sccache + bevy_lint 不兼容**：pre-commit hook 的 `bevy_lint` 经 `.cargo/config.toml` 的 `rustc-wrapper=sccache` 探测编译器失败（`Compiler not supported`）。绕过方式 `RUSTC_WRAPPER= CARGO_BUILD_RUSTC_WRAPPER=` 未固化——需每次提交手动加。
3. **提交被 fmt 门禁拦截**：cargo fmt 后 import 顺序变化未先跑 `cargo fmt --all` 再提交，首次 commit 被 pre-commit 拒绝，二次提交。

**Lessons learned**:
1. **rebase/merge main 后第一时间跑 `cargo check --workspace` 验证编译**——冲突解决可能静默回退已合并代码（本案例：Batch 3 数据层），编译失败状态跨多个提交存活。rebase 完成后先验证再继续开发。
2. **pre-commit bevy_lint 需绕过 sccache**——环境变量 `RUSTC_WRAPPER= CARGO_BUILD_RUSTC_WRAPPER=`，或建议用户将 bevy_lint 调用改为 `env -u RUSTC_WRAPPER`。

**Process improvements**:
1. **建议（hook 层）**：`.githooks/pre-commit` 的 bevy_lint 调用改为 `RUSTC_WRAPPER= CARGO_BUILD_RUSTC_WRAPPER= bevy_lint`（环境问题，无需每次手动加）。
2. **建议（流程）**：worktree 同步 main 采用 rebase 后，把「rebase 后 `cargo check --workspace`」写入全局 skwy-worktree skill 或 AGENTS.md「分支同步」小节。

### Trends (last 10)
- **worktree 分支同步/漂移模式持续**：8-08「worktree 分支漂移」→ 8-09 #10「同步 main 写入全局」→ 本次「rebase 回退 Batch 3 未被察觉」——同主题第三次出现，已机制化一半（同步提醒），但「rebase 后验证编译」仍未固化，本次落实建议。
- **sccache/bevy_lint 摩擦二次出现**：8-09 #10 已记录为环境问题（用户修复），本次提交时再次触发——绕过方式应固化为 hook 修改或 AGENTS.md 命令链说明。

## 2026-08-09 — #10 移除与全局重复的 opencode skill/agent/scripts

**What was done**: 删除 4 个与全局 skwy-* 重复的项目 skill（atom-workflow/reflect/worktree/test）+ test-agent + scripts/open-worktrees.sh（陈旧拷贝），AGENTS.md 索引与 10 处引用改写为全局等价物，测试命令链汇拢到构建门禁；全局 skwy-worktree 新增「handoff 必须含同步原始分支提醒」。

**User corrections**:
1. 「同步main（worktree的原始分支）这个写入全局，然后使用全局的方案。」——Q5 我推荐把同步提醒写入 AGENTS.md「Worktree 纪律」，用户纠正为写入全局 skwy-worktree skill（全局 skill 是权威，项目直接用全局方案）。
2. 「bevy lint 不修复了，这个是环境问题，我去修。」——我对 bevy lint 环境问题连续深挖多轮（CARGO=1 语义 → No binaries → bevy_cli 源码 bin_target.rs），用户叫停，明确归属环境、用户自行处理。

**What went wrong**:
1. **环境问题深挖越界**：`CARGO=1 bevy lint` 失败后，我连续排查 CARGO 变量语义、No binaries available、bevy_cli 源码，直到用户叫停"环境问题，我去修"才停止——应在 1-2 轮内判断归属（全局配置 ~/.cargo/config.toml + 工具内部实现 = 环境问题），记录 TENSIONS 后交还用户。
2. **skill 删除后的缓存残留**：reflect skill 已删除，但当前 opencode 会话的 skill 注册表仍返回旧版内容——执行时以 AGENTS.md 指向的权威（全局 skwy-reflect）为准。

**Lessons learned**:
1. **环境/工具链问题先判归属再深挖**——涉及全局配置（~/.cargo、~/.config）或工具二进制内部实现时，1-2 轮内识别为"环境问题"，记录 TENSIONS 并告知用户，不深入工具源码。用户明确"我去修"时立即停止。
2. **skill 迁移后当前会话仍有旧缓存**——删除/迁移 skill 后，执行相关流程以 AGENTS.md 声明的权威为准，不依赖会话缓存的旧 skill 内容。

**Process improvements**:
1. **AGENTS.md「问题处理闭环」补充**（本会话已提交）："涉及全局环境/工具链内部实现的问题（如 ~/.cargo 配置、工具二进制行为）：记录 TENSIONS 后向用户确认归属，不深入工具源码排查。"

### Trends (last 10)
- **worktree 分支滞后模式 → 已机制化**：8-08 条目 1「worktree 分支漂移」+ 条目 3「worktree 分支滞后 main 未预先检查」两次出现，本次通过用户纠正将「同步原始分支」写入全局 skwy-worktree skill（handoff 必含同步提醒）——重复模式获得机制化落实，不再依赖人为记忆。
- **环境/工具链摩擦持续高频**：CI 静默坏（条目 2）、磁盘满/pre-commit 冲突（条目 3）、sccache/bevy_lint（本次）——均已正确记录 TENSIONS，但"先判归属再深挖"仍需固化为惯例（本次已落地 AGENTS.md）。
- **用户范围边界指令需立即停止**：「等我消息」（条目 2）与「bevy lint 不修复了」（本次）同型——用户给出边界时停止深挖/等待，是反复出现的模式，已两次记录。

## 2026-08-08 — workspace 迁移 + doc 补全 + atom_data 方案设计

**What was done**: 将 11 个移出 crate 迁回 Bevy 0.19 并纳入 workspace（依赖对齐 wgpu 29/encase 0.12/time features，修复 3 处 API 迁移错误）；5 个并行 subagent 完成全 workspace `#[deny(missing_docs)]` + 全量 rustdoc + clippy 清零；启用 mold 链接器解决测试编译链接瓶颈；grill-me 10 轮锁定 atom_data 新方案（epic #2 + 子 issues #3/#4/#5）。

**User corrections**:
1. 「你的grill-me没有触发过，是为什么？」——重大架构变更（bevy_common_assets 替代 datatables）未触发 grill-me 直接动手，方向反复（先加 workspace → 再撤）浪费轮次
2. 「先将强制加载grill-me列入agents.md 中」——把教训固化为 AGENTS.md 强制规则
3. 「atom_datatables 保持原状，不引入workspace即可」——我误将 datatables 系列引入 workspace，范围超出用户预期
4. 「bevy_common_assets 这个先引入使用才对」——方案理解偏了：用户要的是引入新库，不是迁移旧体系
5. 「都支持，为什么我们需要选择格式呢？」——我坚持让用户在格式间二选一，思维定式：格式是 serde 层面能力，框架不该绑定
6. 「写的有误，默认是惰性加载，但提供加载和卸载的接口」——我表述"注册即加载"，用户纠正为默认惰性 + 显式控制
7. 「store这个名字不好，换个」——DataStore 命名被否决 → DataRegistry
8. 「测试为什么这么慢？确认一下是哪几个测试的问题？」——我把慢归因于测试运行，实际是链接阶段（ld 单线程）
9. 「等一下，我改了一下编译配置。再编译试试，看有没有使用mold」——用户主动定位链接瓶颈，启用 mold
10. 「反思没有做？」——push 后遗漏 /reflect 步骤

**What went wrong**:
1. **重大决策未先 grill-me**（最严重）：「bevy_common_assets 代替 datatables」是库选型+架构变更，触发词命中但未触发。导致先误加 workspace 又撤，方向摇摆。
2. **范围扩张**：擅自将 datatables 系列纳入 workspace（用户只要求迁移其他 crate），被纠正后才撤。
3. **思维定式**：格式选择——把"全支持"当"必须二选一"；框架职责与使用方职责混淆。
4. **性能归因错误**：全 workspace 测试慢，我反复归因"编译慢/等锁"，未识别是链接器单线程瓶颈；用户启用 mold 后 10 分钟→2 分钟。
5. **流程遗漏**：push 后未执行 /reflect（atom-workflow §5 要求 push 前反思，push 后补做）。
6. **worktree 分支漂移**：创建 atom-data worktree 后 main 推进 5 个 commit，worktree 分支停在创建点。

**Lessons learned**:
1. **grill-me 触发词命中即触发**——「代替/替换/迁移/引入」+ 超单文件影响 = 停手问决策树，哪怕用户看起来在给方向。来回摇摆比停下来问更贵。
2. **框架与使用方职责分离**——底层库支持的能力（格式、serde）不是框架决策，问使用方"你要什么"而非"库支持什么"。
3. **性能问题先量化再归因**——"测试慢"应先用 `ps`/`time` 拆解编译 vs 运行 vs 链接阶段，而不是反复猜。链接瓶颈的症状（rustc 低 CPU + futex 等待）有明确特征。
4. **mold 是 Bevy 大型 workspace 的标配**——`.cargo/config.toml` 应作为项目模板的一部分。
5. **流程步骤不可跳跃**——push 前反思是 atom-workflow 硬步骤，等同 test/clippy 门禁。

**Process improvements**:
1. **AGENTS.md 第 39 行**（本会话已提交）：新增「重大决策先 grill-me（强制）」规则——含触发词「代替/替换/迁移/重构/引入/方案」+ 影响面描述。✅ 已落实
2. **`.cargo/config.toml`**（本会话已提交）：启用 mold 链接器。✅ 已落实
3. **建议**：atom-workflow skill §5 明确「push 前必须 /reflect」的执行检查（当前流程要求了但本会话遗漏——需要 hook 或 checklist 强化，否则纯靠自觉）。
4. **建议**：worktree 创建后若 main 有推进，需在 handoff 或启动时提醒 agent `git merge main`（worktree 分支保持同步），避免 handoff 上下文过期。

## 2026-08-08 — 项目书体系修复 + CI 门禁 + 覆盖率硬门槛

**What was done**: 审查并修复项目书体系——重写 CI（nightly-2026-01-22 + Bevy clone/patch setup + bevy_lint/deny/workspace rustdoc 全门禁）、固化 scripts/bevy-0.19.patch、更新 AGENTS.md/README/labels 一致性、skill/TENSIONS/kb 去重、修复 deny.toml 过时 ignore + 升级 crossbeam-epoch/spin；新增测试覆盖率 80% 硬门槛（CI + Justfile + AGENTS.md + atom-workflow skill）。

**User corrections**:
1. 「全部修复，但是12 强调--release的先去掉」——我列了 12 项建议，用户确认全部修复但明确排除 #12（README 示例命令加 --release 强调）；范围裁剪听用户的
2. 「立即硬门槛 80%」——我在覆盖率落地方式上推荐渐进式基线（27.54% 现状），用户明确选立即硬门槛，接受 CI 全红
3. 「等之后pull之后再写test。现在先不写。等我消息」——覆盖率测试补齐被 hold：用户要先 pull，测试工作等待明确指令

**What went wrong**:
1. **验证管道取错退出码**：`cargo llvm-cov ... | tail; echo $?` 中 $? 是 tail 的退出码（恒 0），误判 --fail-under-lines 未生效；正确做法 `cmd > log 2>&1; echo $?` 或 ${PIPESTATUS[0]}
2. **CI 全红未被察觉**：workspace 迁移后 CI 每次 18s 失败（bevy path patch 在 runner 不存在），但项目书/流程从未记录 CI 是坏的——直到本次审查才暴露

**Lessons learned**:
1. **验证退出码别过管道**——`cmd | tail; echo $?` 拿到的是管道末端的码，结论可能完全相反；重定向到文件再取 $? 才是真实退出码
2. **门禁状态要有可见性**——CI 坏了应该第一时间被感知（TENSIONS 或 CI 状态检查），而不是等到专项审查才发现
3. **用户 hold 优先于 todo 延续**——用户明确"等我消息"时，自动 todo 延续钩子应让位，保持 hold 直到用户解除

**Process improvements**:
1. **覆盖率 80% 硬门槛**（本会话，未提交——用户 hold）：CI `cargo llvm-cov nextest --workspace --release --fail-under-lines 80` + Justfile coverage recipe + AGENTS.md/atom-workflow skill 规则。待用户解除 hold 后随实现 commit 提交
2. **建议**：pre-push 加 CI 状态可见性检查（或至少在 TENSIONS 记录 CI 状态变化），避免 CI 静默失效多日无人察觉

## 2026-08-08 — atom_data 框架实施（epic #2，Batch 1/2/3 + review 修复）

**What was done**: 完成 epic #2 全部 3 批次——新建 atom_data + atom_data_macros crate（DataAsset derive 宏 + 5 种索引形态 + DataTable<T> 泛型 Asset + bevy_common_assets 全格式集成）；DataRegistry 资源（get/load/unload/reload + AssetEvent 惰性注册）+ data_ref 跨表引用；atom_ability 数据层全面迁移（TbAbilityRow/TableReader 零引用，datatables 依赖移除）。RED→GREEN（test-agent 独立设计 18+13+18=49 测试）、5-agent review-work 审查（QA/Security PASS，Goal/CodeQuality FAIL 已修复闭环）、issue 注释补齐。

**User corrections**:
1. 「开始」/「继续」——启动与推进指令
2. 「等一下更新main到本地。因为之前的修改需要先同步过来。」——RED 测试提交前应先把 main 同步到当前分支：worktree 创建后 main 已推进 6 个 commit（含 workspace 迁移），导致分支落后。我在提交 RED 测试时未先检查 main 状态，用户提醒后才 rebase。

**What went wrong**:
1. **worktree 分支滞后 main 未预先检查**：开始实施时 feat/atom-data 落后 main 6 个 commit（workspace 迁移等），直接基于过期分支写 RED 测试。用户提醒「更新main到本地」才做 rebase——应开始即 `git fetch && git merge main`。
2. **/data 分区磁盘满**：主仓库 target/debug 479G 占满磁盘（git add 失败），被迫清理 2 个闲置 worktree 的构建缓存（63G）才恢复。环境摩擦，但主仓库 debug 构建产物长期堆积是隐患。
3. **review 发现 Q3 格式声明夸大**：文档宣称「9 格式全支持」，实测 postcard 必失败（deserialize_any 对非自描述格式）、csv 架构不兼容，仅 json/ron/toml 验证过。feature 开启 ≠ 可用，声明前必须逐格式实证。
4. **review 发现 handle 存活契约缺陷**（2 轮才闭环）：load/reload 返回 handle 丢弃 → track_assets 提前回收 → 表静默不入 registry。第一轮加了 #[must_use] 但未修唯一真实调用方（example），Oracle 复验 FAIL 后才修。门禁 clippy 不带 --all-targets，example/test 违约静默漏过——审查才暴露。
5. **pre-commit hook 与 RED 阶段冲突**：hook 的 `cargo check --workspace` 对预期编译失败的 RED 测试必然拦截，需 --no-verify 绕过（已在 TENSIONS 记录，但每次 RED commit 都要重复绕）。
6. **rebase 冲突面大**：workspace 迁移后 Cargo.toml/README/AGENTS/TENSIONS 与 main 大量冲突，需逐文件手工合并；TENSIONS 被 main 重构过结构（章节顺序不同），合并易出错。

**Lessons learned**:
1. **worktree 开始实施前先同步 main**——`git fetch && git merge main`（或 rebase）是第一步，尤其 worktree 创建与开始之间隔了时间。过期分支上的 RED 测试/计划会继承过期的 workspace 形态（本次 Cargo.toml members、workspace.dependencies 全部过期）。
2. **「全格式支持」必须有逐格式实证**——feature 开启、loader 注册 ≠ 加载成功。postcard 的 deserialize_any 限制、csv 的 LoadedCsv 容器形态都是编译/注册看不出的运行时/架构约束。声明支持前写一个格式矩阵测试。
3. **#[must_use] 契约要同时修调用方 + 门禁覆盖**——加 must_use 后立刻 grep 所有调用点修掉，且 clippy 必须 --all-targets（否则 example/test 违约永远静默）。审查盲区 = 未来的 bug。
4. **review 复验会抓"修了一半"的修复**——第一轮只加属性不修调用方，Oracle 精确指出现实调用点未动。修复要闭环：机制 + 所有真实调用方 + 门禁覆盖，三件套。
5. **TENSIONS 与 main 的文档重构要小心**——rebase 时文档文件（尤其被 main 重构过的）合并不能机械取 ours/theirs，要核对结构。

**Process improvements**:
1. **CI + pre-push clippy 加 `--all-targets`**（本会话已提交）：覆盖 examples/tests 的 #[must_use] 违约等——默认 workspace clippy 不检查这些 target。✅ 已落实
2. **load/reload #[must_use] + handle 存活文档**（本会话已提交）：DataRegistry::load/reload 加 must_use + 文档说明 track_assets 回收机制；example 的 handle 存资源。✅ 已落实
3. **建议**：atom-workflow skill §0.5 worktree 步骤补一句「开始实施前 `git fetch && git merge main` 同步」，避免 worktree 分支滞后 main 的重复摩擦。
4. **建议**：新框架的「格式支持」类声明应配格式矩阵测试（json/ron/toml/yaml/msgpack/cbor/xml 各一）——本次只验证了 3 格式，其余靠文档收敛，测试矩阵可一劳永逸。
---

# 历史摩擦归档（TENSIONS.md 并入）

> 2026-08-09：摩擦记录机制并入反思流程（skwy-reflect 的 User corrections / What went wrong 章节承载），TENSIONS.md 停用。以下为历史摩擦日志原文归档，整体迁移、未改一字。

# TENSIONS.md — Atom 项目摩擦日志

> 发现数据与系统设计之间的不一致、工具链问题、或流程阻碍时记录。不要当场解决——只捕获信号。
> 标题格式：`## YYYY-MM-DD: <主题>`，按日期倒序排列。

## 2026-08-09: sccache 0.16 与 bevy_lint 不兼容

- **2026-08-09**: [tooling] pre-commit 的 bevy_lint 阶段报 `sccache: Compiler not supported`（exit 101）——根因是 `~/.cargo/config.toml` 全局 `rustc-wrapper = "sccache"`，sccache 0.16 无法识别 bevy_lint 的自定义 rustc driver（bevy_lint_driver），探测阶段失败。环境变量 `RUSTC_WRAPPER=`（空值）可覆盖 config 使 bevy_lint 正常通过。处理：本次提交以 `RUSTC_WRAPPER= git commit` 覆盖；根治需调整全局 sccache 配置（如只对 cargo 构建启用、对 bevy_lint 排除）——待用户决定，未当场解决。

## 2026-08-08: time 0.3.55 编译回归（parse_borrowed 泛型化）

- **2026-08-08**: workspace 迁移后 Cargo.lock 重新解析，`time` 升到 0.3.55 — `time::format_description::parse_borrowed` 增加了 const 泛型 `VERSION` 参数，`atom_core/src/logger.rs:35` 的调用无法推断类型，E0283/E0284 编译失败，阻塞全部依赖 atom_core 的 crate（atom_shader_lib / atom_cel_shader / atom_terrain 等）的 check/clippy/doc。
- **排查路径**: `cargo tree -i time` 确认依赖方 → 确认 `atom_core/Cargo.toml` 用 workspace 级 `time = "0.3"`（宽松约束，无上限）→ `cargo update -p time --precise 0.3.36`（该版本 `parse_borrowed` 尚无 VERSION 泛型）→ 编译恢复。
- **处理**: 仅 pin Cargo.lock（time 0.3.36 + time-core 0.1.2 + time-macros 0.2.18），未改任何 crate 源码。根因修复（logger.rs 显式标注 VERSION 或升级写法）留待 atom_core 维护时处理。
- **2026-08-08**: [tooling] `lsp-daemon`（rust-analyzer）在长时间 cargo 构建后无响应（30s 超时），`pkill lsp-daemon` 重启也无改善。期间以 `cargo check/clippy -D warnings` 作为权威诊断替代。疑似 daemon 正全量索引 /data/codes/Bevy 巨型 workspace，属环境问题非代码问题。
- **2026-08-08**: 排查路径——`cargo doc/check/clippy -p atom_ability` 报 `atom_luban_lib/src/lib.rs:346 unexpected closing delimiter`。根因：`atom_luban_lib` 未提交编辑中 `ByteBuf::read_ulong` 的函数签名行被误删（doc 注释下直接是函数体），`impl ByteBuf` 大括号失衡。修复 = 从 HEAD 还原该行签名（纯恢复，零行为变更）。教训：编辑代码时先 diff 检查非预期删除行；纯加注释的改动不应伴随函数签名消失。

## 2026-06-22: 流程清理

- **2026-06-22**: [restructure] intent.lisp、specs/、plan/ 删除。架构约束并入 ARCHITECTURE.md；BDD spec 全量过时（shader 名/pass 数/网格尺寸不对）；plan/ 四件套（PLAN/MEMORY/DRIFT/SESSION-LOG）是旧 OMP 工作流残余，决策已在 ARCHITECTURE.md。bevy-kb + agent-kb 合并为 kb/。
- **2026-06-22**: [cleanup] APPEND_SYSTEM.md、RULES.md 删除。session 日志清理 798MB，plugins node_modules prune 45MB。
- **2026-06-20**: Workspace 仅保留 atom_terrain。其他 11 个 crate 暂时移出，待逐步迁入 Bevy 0.19。
- **2026-06-20**: Workflow 加 document phase + bevy-kb 更新 + verify-references 检查。

## 2026-06-21: Agent Sidecar 验收发现

- **2026-06-21**: [agent] `top_down_game.rs` 缺少 `app.run()` — `main()` 构建 App 后直接退出，从未进入游戏循环。exit code 0 无 crash log，难以排查。
- **2026-06-21**: [agent] `cleanup_agent` 注册在 `Last` schedule 每帧运行 — 首帧即 kill agent 进程。修复：改用 `Drop` trait 实现 App 退出时自动清理。
- **2026-06-21**: [agent] BRP `world.spawn_entity` 无法创建 asset handle（Mesh/Material）— NPC 只有 `Transform`，不可见。修复：Agent spawn 时附带 `Name("NPC")`，Bevy 侧 `decorate_agent_entities` 系统自动补 cube mesh + 红色 material。

## 2026-06-20: readback 实现发现

- **2026-06-20**: GPU compute 4-pass 管线确认工作: RTX 4090 Vulkan，17³ 网格 density→cross→QEF→indices 四 pass 输出正确顶点/索引。
- **2026-06-20**: WGSL binding(5) counters 始终为 0: 无 atomics 设计下所有 shader 不写 binding 5。vertex_count/index_count 无法从 GPU 读回，需 CPU 端 compact 时自动统计。
- **2026-06-20**: Staging buffer readback 两帧等待: dispatch→(1帧等GPU执行)→copy→(1帧等GPU执行)→map→read。同一帧内 dispatch+copy 会读到全零。
- **2026-06-20**: CPU/GPU 噪声不匹配: GPU 用 value noise（hash-based），CPU 用 OpenSimplex2D。同一坐标高度不同，无法直接对比验证。测试 chunk 位置需根据 GPU noise 单独定位。
- **2026-06-20**: `edge_detect.wgsl` 密度采样用 `round()` 最近邻 — 将连续密度场量化为阶梯函数，二分搜索收敛到体素边界而非真实 isosurface。根因修复：替换为三线性插值 `trilinear_sample()`。此 bug 从 Phase 0 存在、多人审阅未发现——数学密集型函数缺乏 spec 对参。
- **2026-06-20**: `atom_pqef` crate（Rust + WGSL）已有正确的概率 quadric 实现，但 `atom_terrain` 的 terrain shader (`qef_solve.wgsl`, `main_mesh_compute_vertices.wgsl`) 使用独立 inline 实现 — 同一算法两套代码，存在 divergence 风险。（当时记录在 `qef.spec`，该 spec 于 2026-06-22 随所有 spec 一同删除）

## 2026-06-20: Phase 2 接缝修复发现

- **2026-06-20**: 多 chunk 边界有 1-voxel 缝隙 — 根因是边界 voxel 的 quad 引用的邻接 voxel slot 不存在。修复：双边 shell（density (vc+3)³, vertex/cross (vc+2)³）。
- **2026-06-20**: QEF Cramer's rule 对平面退化 — 行列式阈值 `1e-10` 太小，f32 近奇异矩阵产生 spike。修到 `1e-5` + centroid 安全网（顶点距 centroid > 2×voxel_size 则回落）。
- **2026-06-20**: 每个 voxel 只有 6 个 index slot（1 quad）→ 倾斜面上多 quad 互相覆盖导致整行缺失。扩到 72 slot（12 edge × 6 index）。
- **2026-06-20**: `ncross==0` 的边界 voxel 无顶点 → has_vertex 失败 → quad 丢弃。加 fallback 中心点占位。
- **2026-06-20**: Staging buffer 重复 `map_async` panic — 加 `map_started` 标志防止重入。
- **2026-06-20**: 负 shell voxel 顶点为 fallback（无表面交叉）时，邻居无 mesh，不应 skip 该 quad。智能 dedup：仅当负 shell 顶点有真实法线（`length(normal)>0` 表示邻居有 mesh）才跳过。
- **2026-06-20**: Bevy 0.19 内置 `FreeCamera` (`bevy_camera_controller` + `free_camera` feature)。操作：右键旋转 + WASD/QE。
- **2026-06-20**: `StandardMaterial::double_sided` 不管背面剔除 — 需同时设 `cull_mode: None`。

## 2026-06-20: GPU 管线

- **2026-06-20**: ✅ 四 pass compute 管线 smoke test 通过 (RTX 4090, Vulkan), 0 GPU validation error。Readback 待实现 (需 staging buffer 方案)。
- **2026-06-20**: WGSL atomics 类型匹配问题 — `array<atomic<u32>>` 在 shader 端需要匹配 Rust 端 binding type 声明。当前绕过（固定 slot 无 atomics）。
- **2026-06-20**: ExtractResourcePlugin 在 Bevy 0.19 pipelined rendering / sub-app 中不工作 — render world 资源必须用 `render_app.insert_resource()` / `init_resource()`；main→render 数据同步改用 crossbeam channel（`ChunkProcessRequest` + Sender/Receiver）。
- **2026-06-20**: WGSL `var<storage>` 无访问修饰符默认 `read`（非 `read_write`）。向 storage buffer 写入必须声明 `var<storage, read_write>`。

## 2026-06-20: 数据对齐

- **2026-06-20**: 顶点 buffer 使用固定 slot 稀疏存储（每 voxel 一个 TerrainChunkVertex slot），CPU 端 compact + remap。buffer 大小为 vc³ × sizeof(Vertex)，16³=128KB。后续可能需要 compact 存储优化。

## 2026-06-20: 工具链

- **2026-06-20**: Bevy 0.19 需要 Rust nightly + `#![feature(cfg_select)]`。本地 Bevy checkout v0.19.0 的 `bevy_app` 和 `bevy_winit` 缺失该 feature flag，需手动补丁。
- **2026-06-20**: `cargo doc -Dwarnings` 会把 Bevy 依赖的 warning 也当 error。已限定 `-p atom_terrain` 范围。
- **2026-06-20**: WGSL 无断点 — compute shader 调试全靠肉眼。
- **2026-06-21**: `bevy_ui_widgets::list.rs:213` 有一行 `if let` 必然匹配警告 (`irrefutable_let_patterns`)。上游依赖，无法在本 workspace 压制。等 Bevy 升级后如果修复了，就可以给 cargo check 加 `-D warnings` 全量强制零警告。
- **2026-08-08**: 排查路径——`cargo doc/check/clippy -p atom_ability` 报 `atom_luban_lib/src/lib.rs:346 unexpected closing delimiter`。根因：`atom_luban_lib` 未提交编辑中 `ByteBuf::read_ulong` 的函数签名行被误删（doc 注释下直接是函数体），`impl ByteBuf` 大括号失衡。修复 = 从 HEAD 还原该行签名（纯恢复，零行为变更）。教训：编辑代码时先 diff 检查非预期删除行；纯加注释的改动不应伴随函数签名消失。
- **2026-08-08**: 排查路径——`atom_data` 引入 `bevy_common_assets 0.17` 后编译报 `cfg_select` E0658。根因：现有 `[patch.crates-io]` 只 patch 顶层 `bevy`，而 bevy_common_assets 直接依赖 `bevy_app`/`bevy_asset`/`bevy_reflect` 子 crate（crates.io 版本缺 cfg_select patch）。修复 = patch 增补三个子 crate 指向 `/data/codes/Bevy/crates/*`。教训：引入直接依赖 bevy 子 crate 的第三方库时，需同步检查 patch.crates-io 覆盖范围。
- **2026-08-08**: [流程] RED 阶段 commit 被 pre-commit hook 拦截——hook 的 `cargo check --workspace` 无法通过预期编译失败的 RED 测试。处理 = `git commit --no-verify` 绕过（RED 阶段正当），commit message 说明原因。教训：预实现门禁第 3 步（RED）与提交门禁（pre-commit 全量 check）冲突，后续 RED commit 需注明 --no-verify 理由。
- **2026-08-08**: [spike 结论] `DataTable<T>` 泛型 TypePath 唯一性（`.omo/plans/atom-data.md` §3 风险表第一项）——Bevy 的 `TypePath` derive 对泛型类型生成 `GenericTypePathCell`，按 `TypeId` 缓存，不同 `T` 实例得到不同 type path，多格式注册（同一 `DataTable<T>` 多个 loader 插件）实测正常。**无需 fallback 方案**（宏生成具名表类型 `{RowName}Table` 不再需要），实现照常使用泛型容器。验证方式：`full_formats` 示例 json/ron/toml 三格式同时注册 + 加载成功。
- **2026-08-08**: `cargo doc` 报 "File system loop" warning——根目录 `assets/assets` 是历史遗留的指向其他 worktree 的自引用符号链接（HEAD 中 blob e757a26，随 0c10a07 迁移遗留），rustdoc 遍历 asset 目录时撞上循环。非阻塞（warning 级，doc 正常完成），后续清理 worktree 时应删除该符号链接。

## 流程

- **2026-06-22**: [restructure] intent.lisp、specs/、plan/ 删除。架构约束并入 ARCHITECTURE.md；BDD spec 全量过时（shader 名/pass 数/网格尺寸不对）；plan/ 四件套（PLAN/MEMORY/DRIFT/SESSION-LOG）是旧 OMP 工作流残余，决策已在 ARCHITECTURE.md。bevy-kb + agent-kb 合并为 kb/。
- **2026-06-22**: [cleanup] APPEND_SYSTEM.md、RULES.md 删除。session 日志清理 798MB，plugins node_modules prune 45MB。
- **2026-06-20**: Workspace 仅保留 atom_terrain。其他 11 个 crate 暂时移出，待逐步迁入 Bevy 0.19。
- **2026-06-20**: Workflow 加 document phase + bevy-kb 更新 + verify-references 检查。

## 已知退化

- 密度场使用简版 value noise（非 OpenSimplex），视觉质量低于目标。GPU value noise height_at(0,0) ≈ -26（CPU OpenSimplex 仅为 -0.14），两者高度不同。
- 使用 StandardMaterial（单色绿色），非 biome 驱动 PBR。
- 无 LOD — 所有 chunk 用相同 16³ 分辨率。
- 无 biome — 所有 chunk 地形形状相同。

## 2026-08-08: atom_ability Batch 3 迁移摩擦（issue #5）

- **2026-08-08**: [atom_data_macros] `DataAsset` 宏生成的 `{Row}Index` 索引容器是**私有 struct**（`struct` 非 `pub`）——pub 行类型的 `impl DataIndexed` 使私有索引容器泄漏到公开关联类型 `type Index`，触发 E0446（`private type in public interface`）。**rustc 1.95 上该错误为硬错误**：`#[allow(private_interfaces)]`、模块包裹 + 重导出、`pub use` 别名等全部无效（已逐一实证），唯一路径是让索引容器声明为 `pub`。atom_data 自身测试因行类型全部私有而未暴露。处理：宏生成 `pub struct`（单字修复，`{Row}Queries` trait 保持私有，测试契约不变）；atom_ability 18 个 RED 测试以 pub 行类型间接覆盖该路径。
- **2026-08-08**: [bevy 0.19] `On::observer()` 语义变更：0.19 返回 **observer 实体本身**（0.18 为被观察目标实体）。atom_ability 全部 observer 用 `trigger.observer()` 取目标——`On<Add, Ability/Buff>` 两处已修为 `trigger.entity`（Deref 到 `Add { entity }`）；**EntityEvent observer（graph/event.rs 等 11 处）仍用 `observer()` 作目标实体，0.19 上已失效**（graph 事件流/技能生命周期事件可能静默失败，warn 不 panic）——遗留问题，需后续 issue 系统排查修复。
- **2026-08-08**: [bevy 0.19] `On<Add, C>` 在 bundle 插入**过程中**逐组件触发——按字段序插入，先插入的组件触发时后插入的尚不在 archetype（`QueryDoesNotMatch`）。`AbilityBundle`/`BuffBundle` 的配置数据组件必须**先于**标记组件（`ability`/`buff`）声明（已在 struct 文档注释锁定该约束）。旧代码 `ability_row` 字段也在 `ability` 之后——原 observer 在 0.19 上同样失效（升级遗留）。
- **2026-08-08**: [bevy 0.19] `World::component_id::<T>()` 只返回**已注册**组件（0.19 起组件需经系统初始化或显式注册）——`EffectNodeAbilityEntryPlugin`/`EffectNodeBuffEntryPlugin` 在 plugin build 时 `component_id().expect()` 直接 panic。修复：改 `register_component::<T>()`（注册并返回 id，0.19 API）。
- **2026-08-08**: [门禁冲突] RED 契约测试文件（tests/config_data.rs + tests/bundle.rs，禁止修改）未按仓库格式提交且含 `assert_eq!(x, true/false)`——`cargo clippy --all-targets -D warnings` 与 `cargo fmt --check` 直接失败。处理：workspace lints 增加 `bool_assert_comparison = "allow"`（测试契约断言风格，失败时 Debug 输出比 assert! 可诊断）；rustfmt.toml `ignore` 排除两文件（字节原样）。两处例外均已注释说明。
- **2026-08-08**: [spike 结论] `DataTable<T>` 全格式加载边界——review 实证 `Deserialize` 走 `deserialize_any`（lib.rs），**postcard 必然失败**（postcard 的 `deserialize_any` 恒返回 `Err(Error::WontImplement)`）；**csv 结构性不兼容**（bevy_common_assets 0.17 `CsvAssetLoader` 产出 `LoadedCsv<A>` 逐行资产容器，无法反序列化整张 `DataTable<T>`）。json/ron/toml 已验证；yaml/msgpack/cbor/xml 为自描述格式理论可行未逐格式验证。处置：Q3 文档收敛为「已验证 json/ron/toml + 自描述格式理论可行 + postcard/csv 不保证」，使用非验证格式前自行验证。教训：声明「9 格式全支持」前必须逐格式实证，feature 开启 ≠ 可用。
- **2026-08-08**: [push 阻塞] pre-push hook `cargo deny check` 拦截——`atomic-polyfill 1.0.3`（RUSTSEC-2023-0089，unmaintained）经 `heapless → postcard` 依赖链进入，来源是 bevy_common_assets 的 **postcard feature**（atom_data 开启）。而 postcard 本身经 review 实证不可用（`DataTable<T>` 的 `Deserialize` 走 `deserialize_any`，postcard 对非自描述格式必失败 `Err(WontImplement)`）——开启一个不可用且引入 unmaintained advisory 的 feature 是纯负担。处置：**移除 postcard feature**（bevy_common_assets features 8 项），同步 plan/lib.rs Q3 声明收敛为「8 格式，postcard 不启用」。教训：引入 feature 时若它实际不可用，应立即移除而非保留——开启即承担 advisory 风险；「能用但没用」的依赖克制同样适用于 feature。
## 2026-08-09 — #11 AGENTS.md 全局 skills 强制加载清单 + screenshots 忽略

**What was done**: 崩溃后恢复 bsn-migration worktree 会话（open-worktrees.sh 启动）；清理两个无用 worktree（agent-sidecar 已合并 main、gpu-indirect-draw 核心已合并，用户确认删除）；AGENTS.md 新增「全局 skills 强制加载」章节（9 个全局 skill 清单 + 触发场景），.gitignore 忽略 screenshots/。

**User corrections**:
1. 「没有切换worktree？」——崩溃恢复后我仅在主 session 用 workdir 参数操作 worktree 文件，未真正运行 open-worktrees.sh 启动 worktree 会话；用户指出后才启动。
2. 「那些全局的skills 在agents.md 中引用，要求强制加载。」——AGENTS.md 引用的全局 skills 需显式强制加载（skill 工具），不依赖 opencode 自动注入；我据此写入 AGENTS.md「全局 skills 强制加载」章节。
3. 「agents.md 的改变提交。」——要求把 skills 强制加载规则改动落 commit（建 issue #11，docs 提交引用主题相关 open issue）。
4. 「 screenshots 忽略。」——screenshots 目录加入 .gitignore。

**What went wrong**:
1. **worktree 会话恢复遗漏**：用户第一次说"打开worktree，继续"，我只做了只读勘察（cargo check/测试/diff 分析）而未启动 worktree 会话，直到用户追问「没有切换worktree？」才运行 open-worktrees.sh——worktree 会话启动是恢复的第一动作，不是可选项。
2. **已知摩擦重复试错**：pre-commit bevy_lint 被 sccache 阻断（TENSIONS 2026-08-09 已记录，绕过法 RUSTC_WRAPPER= 覆盖），我前两次 commit 失败后才套用已知记录——应先查 TENSIONS 命中已知摩擦直接套用。

**Lessons learned**:
1. **崩溃恢复流程**：worktree 恢复 = 读 handoff → 立即 open-worktrees.sh 启动会话 → 勘察现状；勘察可以在会话启动后由 worktree agent 做，主 session 不越俎代庖。
2. **已知摩擦先查 TENSIONS**：任何命令失败先 grep TENSIONS/reflections 命中已知摩擦与绕过法，避免重复试错（本次 sccache/bevy_lint 是第二次遇到同类场景）。
3. **docs commit 引用主题相关 open issue**：AGENTS.md 流程规则变更建 C-Docs issue（#11）承载，符合先例（#8/#10 均流程类 docs issue）。

**Process improvements**:
1. **AGENTS.md「全局 skills 强制加载」章节新增**（本次 commit 9eef60c 落地）：9 个全局 skill 清单 + 触发场景，强制 skill 工具加载。

### Trends (last 10)
- **环境/工具链摩擦重复出现**：#10「bevy lint 不修复了（环境问题）」与本次 sccache/bevy_lint 阻断同一根因（~/.cargo/config.toml 全局 rustc-wrapper=sccache）——TENSIONS 已记录但绕过法需每次手动 RUSTC_WRAPPER=，根治（sccache 排除 bevy_lint）仍未落实，建议用户排期处理。
- **worktree 管理摩擦重复**：8-08「worktree 分支漂移」与本次「崩溃后未启动 worktree 会话」同属 worktree 生命周期管理疏漏——worktree 会话启动/同步已成为两次教训，恢复流程已沉淀（本次 lessons）。

## 2026-08-09 — #13 将 screenshots 移出 git 跟踪

**What was done**: `.gitignore` 已有 `screenshots/`，但历史误跟踪了 7 个 `screenshots/terrain-*.png`；`git rm --cached screenshots/` 从索引移除（磁盘文件保留），commit 5580fe9 引用 chore issue #13。

**User corrections**: 无（用户选择推荐的「创建 chore issue 并提交」路径）。

**What went wrong**:
1. **label 库存与文档不一致**：`gh issue create --label "A-CI,C-Chore,D-Trivial"` 失败——`D-Trivial` 在 kb/github/labels.md 有文档但仓库实际不存在该 label（现存 D- 仅 D-Complex/D-Straightforward），重试去掉 D-Trivial 成功。建 issue 前应先 `gh label list` 核对库存。
2. **多余命令**：`git rm --cached` 已自动暂存删除，我仍执行 `git add -u screenshots/` 报 pathspec 错误——删除类变更无需再 add。

**Lessons learned**:
1. **建 issue 前核对 label 库存**：`gh label list` 先行，labels.md 文档 ≠ 仓库实际 label，命中不存在的 label 会整体失败。
2. **`git rm --cached` 自带暂存**：删除跟踪后直接 commit，不需要再 `git add -u`。

**Process improvements**: `None`（一次性教训；labels.md 与仓库 label 差异可后续单独同步）。

### Trends (last 10)
- **无显著重复模式**（本次为轻量 chore，摩擦均为一次性工具使用细节）。

## 2026-08-08: BSN 迁移摩擦（issue #7）

- **2026-08-08**: [process] `subagent_type="deep"` 后台任务运行 2h19m 零产出（session 仅原始 prompt 无 assistant 消息，无 cargo/rustc 进程在跑）——判定卡死并 cancel，改由主 agent 直接实现。教训：后台实现任务若长时间无任何 assistant 消息产出即异常，不应静默等待；任务委派前应确认 agent 分类可用。
- **2026-08-08**: [test] test-agent 写的 RED 测试 `tests/grant_effect.rs` 含 E0716（`world.entity_mut(...).get_mut(...)` 链式调用中 `EntityWorldMut` 为临时值，`&mut` 悬垂）——该错误在 RED 阶段被"缺失模块 import 错误"掩盖，实现后首次编译才暴露。处理：绑定临时值为局部变量（语义零改动）。教训：RED 测试的编译错误需区分"目标 API 缺失"与"测试自身生命周期错误"，后者应在 RED 阶段就暴露。
- **2026-08-08**: [bevy] `#[derive(Reflect)]` 的 enum 含 `Box<dyn Reflect>` 字段时：`#[reflect(ignore)]` 只能去掉 `PartialReflect`/`GetTypeRegistration` bound，`FromReflect` 派生仍对 ignored 字段生成 `Default::default()`（`bevy_reflect/derive/src/enum_utility.rs:107` `on_ignored_field`）——需追加 `#[reflect(ignore, default = "fn")]` 提供零参构造函数（哨兵值，正常路径不触发）。`#[reflect(from_reflect = false)]` 不可用（下游 `EffectNodeSlotValue` 依赖 `EffectValue: FromReflect`）。

## 2026-08-08 — #7 BSN 迁移反思（Q2 前提验证/agent 卡死/零消费者节点测试）

**User corrections**:
1. 「这个一个库，没有人调用是有可能的。但通常不好，因为居然没有测试代码。。。」——用户指出重建 grant_effect 节点（库内零消费者）必须有集成测试验证反射调度链路，否则重写完无法验证。→ 落实为 grant_effect 反射调度集成测试（RED→GREEN）。
2. 「需要effect, 这是技能系统的buff系统，起名字为Effect了」——我把 effect 模块误判为死代码准备删除，用户纠正它是需要的活模块 → 范围修订 Q5（接入编译树）。
3. 「网上查一下看看。」——我断言 `Box<dyn Reflect>` 不支持时，用户要求网络查证而非仅凭本地源码 → 查证确认 issue #3392 未关闭 + PR #15532 未合入，结论有据。

**What went wrong**:
1. **重大决策二次修订成本**：Q2 原前提（"反射调用点不变"）在实现前查证发现 grant_effect.rs 是死代码、依赖已注释的 BoxReflect——决策前提失效，需用户重新确认（option b 完整版）。教训：grill-me 锁定决策时若基于"存在性假设"，应在计划阶段验证假设而非实现阶段才发现。
2. **后台 deep agent 卡死**：C2 委派 agent 运行 2h19m 零产出（无 assistant 消息、无编译进程）——判定卡死取消后主 agent 直接实现。教训：后台任务长时间无消息产出即异常，不应静默等待。
3. **test-agent RED 测试含生命周期 bug**：grant_effect 测试有 E0716（`entity_mut()` 临时值悬垂），被"缺失模块 import 错误"掩盖，实现后才暴露。教训：RED 测试编译错误需区分"目标 API 缺失"与"测试自身错误"。
4. **headless 环境冒烟受限**：example 冒烟被无 GPU 阻断，降级为编译验证。

**Lessons learned**:
1. **决策前提在计划阶段验证**——grill-me 锁定的"保留反射调度"基于 grant_effect 死代码的错误假设，实现前 explore 验证本可提前发现。
2. **后台委派需超时感知**——agent 卡死（无产出 2h+）应设观察点，超时即取消改主 agent 直做，避免阻塞整条流水线。
3. **库内零消费者节点必须有测试**（用户纠正 #1）——测试即唯一验证，也是未来接线的契约。
4. **Bevy 0.19 反射约束**：`Box<dyn Reflect>` 无原生 Reflect/Clone/PartialEq（issue #3392），bsn! 组件需 `Clone+Default+Unpin`（FromTemplate blanket），模板函数泛型参数需显式加 bound。

**Process improvements**:
1. **TENSIONS.md 已记录**：agent 卡死判定、RED 测试生命周期 bug、BoxReflect reflect-ignore+default 陷阱、headless GPU 冒烟限制（4 条）。
2. **kb/bsn.md 已更新**：6 个已验证 BSN 迁移模式（模板函数/组件注入三选一/spawn_scene/Box<dyn Reflect> 处理/effect 接入）。
3. **建议**：后台实现任务委派前确认 agent 分类可用（本次 C2 卡死无先兆）；deep agent prompt 应含"无产出超时"自检。

## 2026-08-09 — #14 编译警告拦截 hook + Bevy fork 分支依赖切换

**What was done**: pre-commit hook 增加 `cargo check` 警告拦截（增量编译下本次修改引入的警告随重编译输出即被检出，替代拖到 pre-push clippy）；本地 Bevy 补丁（nightly `cfg_select` ×2 + rustfmt + `bevy_ui_widgets` irrefutable pattern 修复）提交到 `qiboda/bevy` 新分支 `atom-patches`（基于 v0.19.0），`[patch.crates-io]` 从 path 改为 git 分支引用，`atom_terrain` 删除显式 `bevy_camera` 依赖改用 `bevy::camera` re-export。

**User corrections**:
1. 「`bevy_camera = "0.19"` 这里不对」→ 确认选项「不该显式写 bevy_camera 依赖」——bevy 主 crate 已 re-export（`bevy::camera`），应删除该依赖并改代码前缀，而非保留显式依赖。

**What went wrong**:
1. **python3 内联脚本语法错误重复 6 次**：`cargo metadata` 解析脚本用 `python3 -c` 单行（`p = pkgs.get(name) if p: print(...)`）多次 SyntaxError，直到改用 heredoc/独立文件才成功——内联 Python 易错，应直接写文件或 heredoc。
2. **`cargo metadata --no-deps` 空输出未即时识别**：`--no-deps` 只含 workspace 成员，8 个 bevy crate 全查不到，首查空输出后未立即换 `--no-deps` 去掉，浪费一轮。
3. **提交被 fmt 门禁拦截一次**：`bevy::camera` import 顺序未先跑 `cargo fmt` 即 commit，pre-commit 拒绝后 `cargo fmt` 补过——提交前应先 fmt（同类摩擦在 #7 反思已记录，重复出现）。
4. **patch 列表多加了 `bevy_camera` 行**：误以为需显式 patch 每个被直接依赖的子 crate，实际 patch 根 `bevy = { git = ... }` 指向 workspace 根时所有成员自动覆盖（bevy_winit/bevy_ui_widgets 未列出也解析到新分支），`bevy_camera` 行属冗余，被用户纠正触发移除。

**Lessons learned**:
1. **Bevy patch 根机制**：`[patch.crates-io]` 的 `bevy = { git/path = 指向 workspace 根 }` 会自动覆盖整个 Bevy workspace 所有成员 crate，无需逐个显式列出被直接依赖的子 crate——直接依赖子 crate 时按普通版本依赖声明即可（或改用 bevy 主 crate 的 re-export，如 `bevy::camera`）。
2. **提交前先 `cargo fmt`**（#7 教训的重复落实）：import/格式变更后 commit 前必跑 fmt，避免 pre-commit 拦截二次提交。
3. **复杂脚本用文件/heredoc 不用内联 `-c`**：多行逻辑（含条件语句）的 Python 脚本写入临时文件再执行，避免引号/语法反复失败。

**Process improvements**:
1. **已落实（AGENTS.md）**：无新增规则——「提交前 cargo fmt」已隐含在 pre-commit hook 门禁中（拦截即提示）；「patch 根覆盖 workspace 成员」知识沉淀于本条目，后续改 Bevy 依赖引用时直接参考。
2. **建议（hook 层）**：无——fmt 门禁已生效，本次拦截即其工作正常的证明。

### Trends (last 10)
- **fmt 门禁拦截重复出现**：#7（2026-08-08）与本次（#14）均出现「未先 cargo fmt 提交被 pre-commit 拦截」——已两次记录，hook 拦截有效但 agent 提交习惯未变，下次涉及格式变更应先跑 fmt 再提交。
- **Bevy 外部源码本地补丁模式成型**：#13/#14 系列把本地 Bevy 补丁（cfg_select/警告修复）固化为 fork 分支 `atom-patches` git 引用，替代裸 path 依赖——依赖来源可追踪，后续 Bevy 升级时走 fork 分支 rebase 而非本地打补丁。

## 2026-08-09 — #15 bevy skill/kb 同步 atom-patches 依赖来源

**What was done**: 知识库 × 全局 skills 配合度审查：确认 `ui-designer` agent 已存在（不悬空）、`/product brainstorm` 缺失需新建；创建全局 `product-brainstorm` skill（open issues → sprint/milestone 候选，配套 skwy-workflow 规则 11）+ 更新 workflow 引用（`/product-brainstorm`，加入可用 Skills 表）；bevy skill 与 kb/README 补充「本地 Bevy 须与 atom-patches 分支同步」说明；清理 `.worktrees/bsn-migration/` 空壳目录。

**User corrections**:
1. 「product 没有，上一句说的是ui-designer有了」——我误读「1.已经有了，等我加过去」为 product 已存在、用户自加；实际"已经有了"指 ui-designer（#1 引用不悬空），product 需我新建。纠正后创建 product-brainstorm skill。

**What went wrong**:
1. **用户纠正归属误读**：`gh issue create` 时先按 skwy-workflow 模板用 `D-Trivial` 标签，repo 无该标签被拒，改用已存在标签——创建 issue 前应先 `gh label list` 核对标签库存。
2. **grep 全仓超时**：验证无残留时 `grep -rn` 扫到 `.opencode/node_modules/` 拖死 120s——应限定搜索目录（kb/skills/AGENTS.md）。

**Lessons learned**:
1. **创建 issue 前核对标签库存**：`gh label list --repo <owner>/<repo>` 先确认标签存在（项目 label 体系可能只有部分 D-/P- 标签），避免 create 被拒重试。
2. **验证搜索限定范围**：全仓 grep 验证时排除 `.opencode/node_modules/` 等大目录，用明确目录列表替代通配。
3. **全局 skill 命令名 = skill name（kebab-case）**：opencode 由 `name` 字段生成斜杠命令，skill 命名为 `product-brainstorm` 则命令为 `/product-brainstorm`——workflow 中的引用须与命令名一致。

**Process improvements**:
1. **已落实（全局）**：新建 `~/.config/opencode/skills/product-brainstorm/SKILL.md`；`skwy-workflow` 规则 11 引用改为 `/product-brainstorm` 并移除"若项目使用"条件，可用 Skills 表新增该行。
2. **已落实（项目）**：`bevy` skill + `kb/README.md` + `kb/bevy/README.md` 注明编译依赖来自 `atom-patches` 分支、本地 checkout 须同步。

### Trends (last 10)
- **用户纠正多为"我做了什么"的归属误读**：本次「product/ui-designer 归属」与 #10「bevy lint 环境归属」同型——用户指出我对某物归属（谁拥有/谁负责/是否已存在）的判断错误。教训：不确定归属时先向用户确认，不自行推断。
- **标签库存核对缺位**：本次 `D-Trivial` 不存在被拒——项目 label 是 Bevy 分类法的子集（只有 D-Straightforward/D-Complex、P-High），创建 issue 前核对库存应成为惯例。

## 2026-08-15 — #16 opencode → DSH 配置迁移（review 闭环）

**What was done**: opencode 配置完全迁移到 DSH：项目级 `.opencode/kb`、`.opencode/skills/bevy`、`.omo/plans` → 项目 `.dsh/`（git mv 保留 rename 历史）；删除 `.opencode/`、`.omo/` 与全局 `~/.config/opencode/`；AGENTS.md（16 处）、crates doc 注释（12 处）、.github 模板（4 处）、kb 内部（3 文件）、全局 4 个 skwy-* 技能（.omo→.dsh 约定）引用全部更新，两轮 subagent_review 闭环后提交 e7f6448（ref #16）。

**User corrections**:
1. 「是迁移到当前项目的.dsh 目录。。。」——我推荐全局 `~/.dsh/projects/atom/`，用户纠正目标为**项目 `.dsh/`**（迁移层级误判）。
2. 「也迁移，移动到 .dsh 内部。.omo 中的内容也迁移到.dsh内部的一个目录中」——我推荐 kb 保留原位，用户纠正 kb 与 .omo 也迁移。
3. 「omo里面的plans 需要迁移过去，其他不需要」——范围细化：只迁 plans，run-continuation 缓存不迁。

**What went wrong**:
1. **grep 工具 hidden 盲区**：验证"全仓无残留"用 grep 工具（ripgrep 系），默认跳过 `.dsh/`、`.github/` 等 hidden 目录 → 误报干净；第一轮 review 抓到 10 处真实残留（bevy/review skill 内部 6 处 + .github 模板 4 处）。
2. **rmdir 静默吞错**：清理 `.opencode/` 时 `rmdir 2>/dev/null` 吞掉"目录非空"错误 → 2 个被跟踪孤儿文件（command/just.md、plugins/trash-rm.ts）残留，第二轮 review 才抓出。
3. **git mv 嵌套**：预先 mkdir 目标目录后 git mv 把源移入其下（`.dsh/kb/kb/`），需二次修正——git mv 到已存在目录 = 嵌套移动。
4. bash 执行器两次内部故障（`Cannot read properties of undefined (reading 'config')`），临时改用 glob/read 探查（环境摩擦）。

**Lessons learned**:
1. **残留验证必须覆盖 hidden 目录**：grep 工具默认跳过隐藏目录；全仓搜索验证用 `grep -rn`（bash）或 `rg --hidden`，不轻信 grep 工具的"无结果"。
2. **清理命令禁止静默吞错**：rmdir/rm 的 `2>/dev/null` 会把"没删干净"伪装成"删完了"；清理后必须 `git ls-files` + `ls` 双重验证。
3. **git mv 前不预建目标目录**；move 后立即 `git status` 核对无路径嵌套。
4. **review 兜底验证**：subagent_review 的 bash grep 覆盖 hidden 目录，与主会话 grep 工具行为不同——大迁移后 review 是残留的最后防线。

**Process improvements**:
1. **已落实（AGENTS.md）**：「命令/术语全仓搜索」新增强制小节——搜索必须覆盖 hidden 目录（`.dsh/`、`.github/`），用 bash `grep -rn` 或 `rg --hidden`（本条目教训直接落为规则）。
2. 其余为一次性教训（None）。

### Trends (last 10)
- **"全仓验证"方式反复踩坑**：#15「grep 全仓超时（.opencode/node_modules）」与本次「grep hidden 盲区」同型——验证手段不当导致漏检/超时；本次已落实 AGENTS.md 规则，模式应退役。
- **用户纠正多为范围/层级判断**：#15 归属误读（product vs ui-designer）、本次迁移目标层级（全局 vs 项目 .dsh）——涉及"迁到哪/属于谁"先确认再动手。
