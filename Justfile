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
