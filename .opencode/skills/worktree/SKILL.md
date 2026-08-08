---
name: worktree
description: 管理 PR/功能分支开发的 git 工作树。用于创建、列出或删除 .worktrees/ 下的工作树。当用户说 "worktree"、"切一个worktree" 或需要 PR 工作空间时触发。
---

# Worktree

Git 工作树为 PR/功能分支开发提供隔离的工作目录。每个工作树是单个功能分支的**临时工作空间**——开发开始时创建，合并后删除。

## 约定

所有工作树位于 `.worktrees/<name>/` 下（已 gitignore）。分支命名：`feat/<short-description>` 或 `fix/<short-description>`。

```
.worktrees/
├── feature/gpu-indirect-draw/   # 多模块 GPU 功能
└── feature/agent-sidecar/       # Agent sidecar 集成
```

| 工作树路径 | 用途 |
|---|---|
| `.worktrees/<name>` | 临时功能分支工作空间——每个功能分支一个 |

## 创建时机（MANDATORY）

**多模块 feature/epic 类工作（2+ 模块、将产出设计/计划文档）时创建并切换；单文件修复/纯文档不需要。** 判断口诀：一旦确定"这次要产出设计/计划文件"→ 先开 worktree 再写文件。

**注意**：git worktree 是独立 checkout——**master 工作区的 untracked 文件不会出现在 worktree 中**。禁止在 master 工作区先产出 plan/design 再等开 worktree 迁移。

## 命令

### 创建

```bash
# 从 main 切功能分支（默认场景）
git worktree add -b feat/<name> .worktrees/<name> main

# 从目标分支切修复分支（场景：修复特定功能分支的构建/测试）
git worktree add -b fix/<name> .worktrees/<name> <target-branch>
```

**规则**：
- `<name>` = 与功能匹配的 kebab-case 短名（例如 `feature/gpu-indirect-draw`）
- 默认基于 `main`——功能合并回 main
- 绝不在 `.worktrees/` 之外创建工作树

**创建后**：
1. 在 `.worktrees/<name>/` 内进行全部实现、验证、提交（在 worktree 目录中运行 cargo 命令）
2. 功能完成后合并回 main：`git checkout main && git merge feat/<name>`
3. 清理：`git worktree remove .worktrees/<name> --force && git branch -D feat/<name>`

### 列出

```bash
git worktree list
```

### 删除（合并后）

```bash
git worktree remove .worktrees/<name> --force
git branch -D feat/<name>
```

### 清理孤立目录

删除 `.worktrees/` 下不是活跃 git 工作树的目录：

```bash
for d in .worktrees/*/; do
  name=$(basename "$d")
  if ! git worktree list | grep -q ".worktrees/$name"; then
    echo "orphan: $d"
    rm -rf "$d"
  fi
done
```

## 纪律（强制）

- **worktree 一旦创建，后续实现工作必须在 worktree 内完成**；master 上不再继续实现
- **master 只允许 docs/lint/typo/反思类提交直推**
- 存在活跃 worktree 时实现类提交落在 master 即流程违规，在 `.opencode/kb/TENSIONS.md` 记录
- 开始实现前用 `git worktree list` + `git branch --contains HEAD` 确认所在分支；不确认分支归属就不开始

## 示例

```bash
# 用户："切一个 GPU indirect draw 的 worktree"
git worktree add -b feat/gpu-indirect-draw .worktrees/gpu-indirect-draw main
# → 在 .worktrees/gpu-indirect-draw/ 内实现
# → 合并后清理
git worktree remove .worktrees/gpu-indirect-draw --force
git branch -D feat/gpu-indirect-draw
```
