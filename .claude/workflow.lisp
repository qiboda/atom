;; Atom Development Loop — workflow.lisp
;; 每次实现任务的强制流程。不可跳过环节。
;; 灵感: Khairold plan-protocol + agent-spec lifecycle + SOUL.md 边界

(workflow atom-dev-loop
  (invariant "每个环节完成前不可跳转；被阻塞时显式声明缺失信息及已尝试路径")

  ;; ── Phase-Gate Protocol ──
  ;; 进下一阶段前 MUST 发一行 gate 声明:
  ;;   [GATE <from>→<to>] <exit-condition 逐条对账结果>
  ;; 禁止: 编译通过就默默进入 verify; 报错修复就退回 implement
  ;; 缺失 gate → 流程违规，必须回溯补发

  (phase understand
    (purpose "理解需求，消除歧义")
    (actions
      (read-spec "读取 specs/*.spec 获取验收标准和边界")
      (check-constraints "对照 SOUL.md 检查: 是否违反依赖规则/架构边界/不可触碰项")
      (ask-before-guessing "需求模糊且无法从代码/文档推断时，用 ask 给出 2-4 个互斥选项 + 推荐默认值"))
    (exit-condition "可以一句话描述: 要改什么、为什么、验收标准是什么"))

  (phase research
    (purpose "发现可复用代码、现有模式、影响范围")
    (actions
      (find-existing "搜索项目中是否有类似实现或可复用工具")
      (trace-callers "对要修改的导出符号，用 lsp references 找所有调用点")
      (read-before-edit "修改文件前必须 read 该文件的目标区域，禁止凭记忆编辑")
      (identify-patterns "定位要复用的现有模式，禁止引入第二种惯例做同一件事"))
    (exit-condition "知道现有代码的准确位置、调用关系、要复用的模式"))

  (phase design
    (purpose "确定方案，消除实现者的所有设计决策")
    (actions
      (pick-approach "基于 research 的发现选一个方案，简述取舍，然后锁定 — 不列出 Alternatives")
      (define-steps "按行为分组编排步骤，每步写清: 具体编辑 + 目标文件/符号 + 新代码的理由")
      (handle-edges "每个新路径指明空/缺失/冲突/错误的处理方式，或声明不需要")
      (list-critical-files "列出 ≤5 个消歧义的关键文件锚点"))
    (exit-condition "另一位工程师不看对话原文、不做任何设计决策就能按步骤实现"))

  (phase document
    (purpose "为设计产物编写 rust-doc，描述项目现状和 API 契约")
    (format "/// 文档注释集成进代码，使用 Rust RFC 1574 规范")
    (sections "模块级 (#![deny(missing_docs)] + //! module doc) → 类型级 (/// struct/enum doc) → 方法级 (/// fn doc, 含 # Summary / # Examples / # Panics)")
    (rule "spec 定义验收标准和边界; rust-doc 描述 API 用法和当前实现行为。两者互补，不冲突")
    (rule "文档 MUST 先于实现: 写文档 = 锁定 API 契约。实现偏离文档 → 更新文档而非反过来")
    (enforce "公共 API 强制 #[deny(missing_docs)]; 非公共辅助按需加 // 普通注释")
    (exit-condition "cargo doc --no-deps 零警告; 新开发者仅读 rust-doc 即可理解和使用 API"))

  (phase implement
    (purpose "按文档和设计逐步编码，每步保持编译通过")
    (actions
      (work-stepwise "按 design 的步骤顺序执行，每步完成后立即 cargo check")
      (no-dead-code "删除旧代码，禁止留下兼容别名、注释掉的旧实现、// TODO")
      (fix-at-source "修复问题的根源，不抑制警告、不特殊处理输入")
      (follow-conventions "中文注释非平凡逻辑，禁止 unwrap()，用 expect(\"原因\")")
      (update-callers "改签名就迁移所有调用点，不留兼容层")
      (max-retries "同一 design 步骤编译失败 ≤ 3 轮; 超过则回退到 design 重审")
      (update-docs "添加新公共 API 时同步写 rust-doc; 修改签名时同步更新文档"))
    (exit-condition "所有步骤完成，cargo check 通过。不发 gate 声明禁止离开此阶段"))

  (phase verify
    (purpose "证明改动端到端工作")
    (actions
      (lint "cargo clippy && bevy_lint — 零新警告")
      (test "cargo nextest — 至少运行被改动代码的相关测试")
      (smoke "运行受影响的示例或集成场景，验证新行为可观测")
      (inspect-output "确认输出是预期的，不是仅凭 build/typecheck 就声称通过"))
    (exit-condition "至少一项端到端检查通过，直接观察新行为生效。不发 gate 声明禁止离开此阶段"))

  (phase review
    (purpose "清理和记录")
    (actions
      (commit "git commit 前确保 pre-commit hook 通过")
      (no-leftovers "检查无遗留的调试打印、注释掉的测试、未使用的 import")
      (update-docs "如改动改变了公共 API 或惯例，同步更新 intent.lisp 和 APPEND_SYSTEM.md"))
    (exit-condition "工作区干净，commit message 描述准确"))

  ;; ── Common Failure Modes (已发生过的流程违规) ──

  (anti-pattern implement-sprawl
    (symptom "cargo check → 报错 → 修复 → 再 check 无限循环")
    (cause "隐式子任务淹没阶段边界感知; 工程师进入 tunnel vision")
    (hard-rule "同一 design 步骤编译失败超过 3 轮 → 回退到 design 重审方案"))

  (anti-pattern verify-skipping
    (symptom "cargo check 通过后直接 commit，没有 gate 声明")
    (cause "把 '编译通过' 等同于 '做完了'")
    (hard-rule "cargo check 通过后 MUST 发 '[GATE implement→verify]' 然后依次跑 lint/test/smoke"))

  (anti-pattern phase-drift
    (symptom "在 implement 中做 verify 的事，或在 verify 中重构")
    (hard-rule "每个 phase 只做该 phase 的事。发现遗漏 → 退回去完成; 提前的事不做"))

  (anti-pattern doc-skipping
    (symptom "设计完成后直接 implement，没有写 rust-doc")
    (cause "把 spec 当成文档用，或认为 '文档可以后补'")
    (hard-rule "design 之后 MUST 经过 document phase。无 cargo doc 检查的 implement 视为违规"))

  ;; ── Supporting protocols ──

  (habit log-before-fixing
    (purpose "捕获摩擦信号，不跳过诊断直接修复")
    (rule "遭遇架构不一致、工具链问题、或流程阻碍时，先记录到 .claude/TENSIONS.md 对应分类下")
    (categories (gpu-pipeline data-alignment toolchain workflow known-degradation)))

  (blocked-protocol
    (when-stuck "准确陈述缺失信息 + 列出已尝试的获取路径")
    (keep-working "如果可继续做其他部分，先做那些; 禁止因一项阻塞就停止所有工作"))

  (test-strategies
    (purpose "按代码层级选择验证策略")
    (pure-rust "红绿 TDD: #[test] + cargo test")
    (ecs-system "集成测试: bevy::app::App 无窗口")
    (gpu-compute "手动验证: example + 肉眼 + 记入 TENSIONS.md"))

  ;; ── Quality Gates ──

  (quality-gates
    (per-step "cargo check 必须在每步编辑后通过")
    (pre-commit "cargo check + cargo doc --no-deps + bevy_lint (fast)")
    (pre-push "cargo check + cargo clippy + bevy_lint + cargo nextest + cargo doc --no-deps (full)")
    (after-commit "spec guard 非阻塞运行")))
