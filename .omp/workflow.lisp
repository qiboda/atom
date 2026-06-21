;; Atom Development Loop — workflow.lisp
;; 精简版: 不变式 + 反模式 + 质量门。
;; 完整 phase checklist 见 AGENTS.md "操作协议"。
;; 测试策略见 SOUL.md。

(workflow atom-dev-loop
  ;; ── 不变式 ──
  (invariants
    "每个 phase 完成前不可跳转；阻塞时显式声明缺失信息及已尝试路径"
    "快速任务(typo/import/纯重构)可豁免 understand→design→document，但仍需 implement→verify→review"
    "所有公共 API 强制 #[deny(missing_docs)] + rust-doc RFC 1574"
    "不引入第二种惯例做同一件事；复用现有模式"
  ;; ── 已发生的流程违规 ──
  (anti-patterns
    (implement-sprawl
      "cargo check → 报错 → 修复 无限循环"
      "≥3 轮编译失败 → 回退 design 重审方案")
    (verify-skipping
      "cargo check 通过后直接 commit"
      "MUST 跑 cargo clippy + cargo test")
    (phase-drift
      "在 implement 中做 verify，或在 verify 中重构"
      "只做当前 phase 的事；遗漏→退回，提前→不做")
    (doc-skipping
      "design 后直接 implement 无 rust-doc"
      "design 后 MUST 经过 document phase")
    (doc-rot
      "元文档(intent.lisp/APPEND_SYSTEM/TENSIONS) 描述已删除的旧架构"
      "review phase MUST 执行 verify-references + update-kb + update-docs"))

  ;; ── 支持协议 ──
  (protocols
    (log-before-fixing
      "遭遇架构不一致/工具链问题/流程阻碍 → 先记入 TENSIONS.md 对应分类"
      (categories gpu-pipeline data-alignment toolchain workflow known-degradation))
    (blocked
      "准确陈述缺失信息 + 列出已尝试获取路径"
      "其他部分可继续 → 先做那些；不因一项阻塞停止所有工作"))

  ;; ── 质量门 (与 .githooks/ 同步) ──
  (quality-gates
    (per-step "cargo check")
    (pre-commit "cargo check + cargo doc --no-deps + bevy_lint + agent-spec guard (guard non-blocking)")
    (pre-push "cargo check + cargo clippy + cargo doc --no-deps + bevy_lint + cargo nextest (full)")
    (after-commit "spec guard 非阻塞运行（pre-commit 未覆盖时补充）"))

  ;; ── 回顾 ──
  (reflect
    "每轮 review 后执行: 回顾全过程 → 识别流程/工具/架构不足 → 产出 ≤5 条 actionable → 写入 SESSION-LOG"
    (triggers "哪里走了弯路？哪些信号被忽略？流程和工具哪里卡了？架构是否有重复/不一致？")
    (format "## YYYY-MM-DD — reflect: <task>" (items "what-went-well" "what-was-hard" "surprising" "improvements"))))
