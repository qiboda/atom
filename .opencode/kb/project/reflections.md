# 反思日志

> 实施后反思沉淀。只追加，不修改/删除历史条目。格式见 `reflect` skill。

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