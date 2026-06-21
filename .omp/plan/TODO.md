# TODO — GPU DC 地形管线 → 游戏框架

基于 PLAN.md，当前工作聚焦在 GPU DC 地形管线收尾和 Agent 集成验证。

## 阶段 A：管线收尾

- [x] **合并 GPU indirect draw** — `.worktrees/feature/gpu-indirect-draw` 审查后合并
- [ ] **消除 bevy_ui_widgets 编译警告** — 等 Bevy 升级后消掉

## 阶段 B：Agent 集成

- [ ] **Agent 日志游戏内显示** — HUD 控制台，实时显示 agent 日志 [backlog]
- [ ] **Agent 输入/交互** — 按键打开输入框召唤 agent
- [ ] **丰富游戏事件** — 把更多游戏事件 push 到 agent（item pickup、NPC 交互等）

## 阶段 C：游戏内容

- [ ] NPC 交互基础（对话、交易）
- [ ] 物品系统（拾取、背包）
- [ ] Biome 分布与纹理
- [ ] TerrainMaterial（biome 驱动 PBR）
- [ ] 寻路 (nav mesh)
- [ ] 技能系统与地形交互
- [ ] CSG 洞穴/构造物生成

## 阶段 D：工程

- [ ] 分支模型约定（feature/fix/hotfix 前缀、PR merge 策略）
- [ ] CI 冒烟测试（截图 diff）
- [ ] GPU indirect draw 最终确认（移除 `do_readback` → `build_global_mesh` 路径）

---

**说明**: `[backlog]` 标记的暂不实现，等前置条件满足或资源就绪后激活。
持续更新：完成一项就勾一项。
