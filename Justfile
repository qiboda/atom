# Atom 项目任务编排
# 用法: just <task>

# 快速检查（编译，不生成二进制）
check:
    cargo check --workspace

# Clippy 检查
clippy:
    cargo clippy --workspace

# 完整测试
test:
    cargo test --workspace

# agent-spec 命令
# 初始化新 spec
spec-init name:
    agent-spec init --level task --lang zh --name "{{name}}"

# Spec 质量检查
spec-lint spec:
    agent-spec lint specs/{{spec}}.spec

# Spec 生命周期验证
spec-lifecycle spec:
    agent-spec lifecycle specs/{{spec}}.spec --code .

# 全 spec 守卫 (提交前)
spec-guard:
    agent-spec guard --spec-dir specs --code . --change-scope staged

# Spec 审查摘要
spec-explain spec:
    agent-spec explain specs/{{spec}}.spec --code . --format markdown

# 构建所有 release
build:
    cargo build --workspace --release

# 格式化代码
fmt:
    cargo fmt --all

# 运行地形查看器 (release)
run:
    cargo run -p atom_terrain --example chunk_loader --release

# 运行地形查看器 (debug, 有更多日志)
run-debug:
    cargo run -p atom_terrain --example chunk_loader

# CI: 全量检查
ci: check clippy test
