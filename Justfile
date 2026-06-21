# Atom 项目任务编排
# 用法: just <task>

# 列出所有可用命令
commands:
    @just --list

# 快速检查（编译，不生成二进制）
check:
    cargo check --workspace

# Clippy 检查
clippy:
    cargo clippy --workspace

# Bevy linter 检查 (需要 bevy_lint v0.6.0 + nightly-2026-01-22)
bevy-lint:
    $HOME/.cargo/bin/bevy_lint

# 测试 (nextest 并行执行)
test:
    cargo nextest run --workspace

# agent-spec 命令
# 初始化新 spec
spec-init name:
    agent-spec init --level task --lang zh --name "{{name}}"

# Spec 质量检查
spec-lint spec:
    agent-spec lint .omp/specs/{{spec}}.spec

# Spec 生命周期验证
spec-lifecycle spec:
    agent-spec lifecycle .omp/specs/{{spec}}.spec --code .

# 全 spec 守卫 (提交前)
spec-guard:
    agent-spec guard --spec-dir .omp/specs --code . --change-scope staged

# Spec 审查摘要
spec-explain spec:
    agent-spec explain .omp/specs/{{spec}}.spec --code . --format markdown

# 生成并打开 workspace 文档（不含依赖）
doc:
    cargo doc --no-deps --open --workspace

# 构建所有 release
build:
    cargo build --workspace --release

# 格式化代码
fmt:
    cargo fmt --all

# 依赖审计 (license + 重复依赖 + 安全漏洞)
deny:
    cargo deny check

# 运行地形示例 (release)
run:
    cargo run -p atom_terrain --example chunk_loader --release

# 运行地形查看器 (debug, 有更多日志)
run-debug:
    cargo run -p atom_terrain --example chunk_loader

# CI: 全量检查
ci: check clippy bevy-lint test
