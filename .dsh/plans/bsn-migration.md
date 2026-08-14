# BSN 迁移计划 — atom_ability Bundle → BSN (issue #7)

> 本计划在 worktree `feat/bsn-migration` 内创建，随实现 PR 提交。
> 上下文契约：`.dsh/plans/handoff.md`（grill-me 锁定决策）+ 本计划（Q2 修订版）。

## 1. 目标

`atom_ability` 全面从 `Bundle` 迁移到 BSN（Bevy Scene Notation，Bevy 0.19）：
删除所有 `#[derive(Bundle)]` 结构体和 `BundleTrait`/`AbilityBundleTrait`/`BuffBundleTrait`，
改为 `bsn!` 模板函数 + 组件内联；**保留并重建 `EffectBundleTrait` 反射调度**（Q2，option b 完整版）。

## 2. 决策记录（grill-me 锁定 + 本次修订）

| # | 决策 | 状态 |
|---|---|---|
| Q1 | **B** 全面删除 `#[derive(Bundle)]` + `BundleTrait`/`AbilityBundleTrait`/`BuffBundleTrait` | 锁定 |
| Q2 | **A→修订** 保留反射 trait 调度——`EffectBundleTrait` 从 `spawn_bundle` 改 `spawn_scene` 语义；**在新架构重建 grant_effect 节点**（option b 完整版，用户 2026-08-08 确认） | 修订 |
| Q3 | **A** 模板函数承载构造逻辑：`XxxBundle::new(data)` → `fn spawn_xxx(data) -> impl Scene { bsn!{...} }` | 锁定 |
| Q4 | **A** 反射 trait 返回 `EntityCommands`：`fn spawn_scene(&self, commands) -> EntityCommands`，内部 `commands.spawn_scene(self.build_scene())` | 锁定 |

### Q5 范围修订（2026-08-08，用户确认）

**effect 模块（buff 系统效果层）本次一并接入编译树**：
- `src/effect/*` 是 bevy_ability fork 遗留的完整效果子系统（EffectState 状态机、EffectTime 计时、
  效果标签容器、EffectGraphMap），**从未在新图架构下编译**（lib.rs 无 `pub mod effect`）。
- 它引用已不存在的旧图事件（`EffectNodeStartEvent`/`EffectNodeEvent`/`EffectNodeCheckStartEvent`）
  和过时的 `atom_layertag::container::` API（现在 `container_op::`）。
- 本次范围：接入 `lib.rs` 编译树 + 迁移 effect 的 Bundle + **重写 effect 对图事件的交互**
  （旧事件 → 新架构 `EffectGraphExecEvent`/`EffectNodeExecEvent`/入口节点语义）。
- 依赖矩阵查证见 explore 报告，接入细节待实现阶段落实。

### Q6 effect 接入新设计点（2026-08-08，用户确认）

| # | 决策 |
|---|---|
| E1 | **新建 `EffectNodeEffectEntry`**——仿 `EffectNodeAbilityEntry`/`EffectNodeBuffEntry`（Component + `impl_effect_node_pin_group!` + Plugin 注册 `TypedComponentIds`），输出 exec 口 `start`/`ready`/`abort`/`end`；effect 激活流程 `Active` 时 `commands.trigger(EffectGraphExecEvent { entry_exec_pin: start, ... })` |
| E2 | **graph_class 继承 buff/ability 的数据表行**——effect 实体的图类别从所属 buff/ability 行继承，不新增 effect 数据表字段 |
| E3 | **到时触发图 END 事件收尾**——`time_end_destroy_effect` 仿 `buff/timer.rs:52-59`：到时 `commands.trigger(EffectGraphExecEvent { entry_exec_pin: end, execute_in_graph_state: Some(Active), ... })`，图完成后经 ToRemove 流程清理，不直接 despawn |

### Q2 修订依据（已查证）

1. `src/graph/base/*` 是死代码：`graph/mod.rs` 无 `pub mod base`，文件引用不存在的
   `EffectBundleTrait`/`EffectNodeBaseBundle`/`EffectNodeUuid`/`EffectNodePendingEvents`。
2. `EffectValue::BoxReflect(Box<dyn Reflect>)` 曾在 blackboard.rs:51 被注释（`// TODO: add when bevy support`）。
   **网络查证**：Bevy 0.19 **不支持** `Box<dyn Reflect>` 原生反射——
   - `impl Reflect for Box<dyn Reflect>` 全仓搜索 0 命中；issue #3392（2021）至今 OPEN；
     PR #3400/#9929/#14776/#15532 均未合入 0.19。
   - 但 `#[reflect_trait]` + `TypeRegistry::get_type_data::<ReflectXxxTrait>` + `.get(&dyn Reflect)`
     **是官方成熟模式**（`examples/reflection/dynamic_types.rs` 官方示例）。
3. 方案：`EffectValue::BoxReflect(#[reflect(ignore)] Box<dyn Reflect>)` + **手写 `Clone`/`PartialEq`**
   （`Box<dyn Reflect>` 无 std impl）；`#[derive(Debug)]` 保留。

### 架构不变量（不得破坏）

- 效果图动态调度 = `EffectGraphBuilderMap`（builder trait map 按 `GraphClass` 查图模板）+ 反射 trait 调度（grant_effect）。
- 新增效果实体由 `EffectNodeExecEvent`（输入执行口）+ `EffectGraphExecutor`（推进链）+ `EffectNodeId`（Entity/Uuid）驱动——即新架构，非旧 pending-events 模型。

## 3. 技术验证（Bevy 0.19 源码确认）

- `bsn!` 支持 `{expr}` 外部变量捕获、`template_value(component)` 注入、`impl Scene`/`SceneList` 模板
  （`/data/codes/Bevy/crates/bevy_scene/src/lib.rs:480-560`）。
- `Commands::spawn_scene(scene) -> EntityCommands`（`spawn.rs:263`，`CommandsSceneExt`）。
- `Scene: SceneBox: Send + Sync + 'static`（`scene.rs:48`）；`template_value<T: Template>` 要求
  `Clone + Default + Unpin`（`bevy_ecs/src/template.rs:390` blanket impl）。
- `#[reflect_trait]` 生成 `ReflectXxxTrait { get(&dyn Reflect) }`（`bevy_reflect/derive/src/trait_reflection.rs`）。
- `impl_effect_node_pin_group!` 用 `TypeId::of::<$in_type>()` → `Box<dyn EffectBundleTrait>` pin 可行。

## 4. 涉及文件与迁移方式

### 4.1 删除（死代码/废弃体系）

| 文件 | 处理 |
|---|---|
| `src/bundle.rs` | 删除 `BundleTrait`/`AbilityBundleTrait`/`BuffBundleTrait`；**保留文件，重写为 `EffectBundleTrait`（Q2/Q4）+ `ReflectEffectBundleTrait`** |
| `src/graph/base/*`（entry.rs, grant_effect.rs, log.rs, multiple.rs, timer.rs, mod.rs） | 全部删除（死代码，未声明 `pub mod base`）；grant_effect 逻辑按新架构重建（见 4.4） |
| `src/effect/bundle.rs` | `EffectStartTagBundle`/`EffectAbortTagBundle` 迁移为组件内联（effect 模块接入编译树，见 §4.7） |
| `src/graph/bundle.rs` | 删除 `EffectGraphBundle`（组件内联，见 4.5） |
| `src/graph/node/bundle.rs` | 删除 `StateEffectNodeBundle`；**保留 `InstantEffectNodeBase`**（live，被 implement/log.rs、seq.rs 使用）→ 移至 `node/mod.rs` |

### 4.2 ability 侧 → bsn! 模板

| 旧 | 新 |
|---|---|
| `ability/bundle.rs::AbilityBundle`（`new()` 构造） | `ability/bundle.rs::spawn_ability(ability_row, state_registry) -> impl Scene`，`bsn!` 内联 6 个组件 + `template_value()` 注入 4 个 layertag 容器 |
| `ability/bundle.rs::AbilityOwnerBundle<T>` | `spawn_ability_owner::<T>() -> impl Scene`（`bsn!{ T StateLayerTagContainer }`，`T: AttributeSet + Component + Default`） |
| `ability/layertag/bundle.rs::AbilityStartTagBundle`/`AbilityAbortTagBundle` | 逻辑并入 `spawn_ability` 模板函数体（或保留为纯组件，`new()` 逻辑移入模板） |
| `examples/ability/main.rs` | `commands.spawn_scene(spawn_ability(...))` |

### 4.3 buff 侧 → bsn! 模板

| 旧 | 新 |
|---|---|
| `buff/bundle.rs::BuffBundle`（`new()`） | `spawn_buff(buff_row, state_registry) -> impl Scene`（组件内联 + `template_value`） |
| `buff/layertag/bundle.rs::BuffStartTagBundle`/`BuffAbortTagBundle` | 逻辑并入 `spawn_buff` |
| `buff/event.rs`（`commands.spawn(buff_bundle)`） | `commands.spawn_scene(spawn_buff(...)).set_parent_in_place(...)` |

### 4.4 grant_effect 重建（Q2 核心交付）

**位置**：`src/graph/base/` 删除后，按新架构在 `src/graph/node/implement/grant_effect.rs` 重建
（与 log/timer/seq 同层），随 `graph/mod.rs` 声明 `pub mod base`（新 `base` = 重建节点集）
或直接放 `node/implement/`。

**组件**：`EffectNodeGrantEffect { effects: Vec<Entity> }`，`impl_effect_node_pin_group!`：

```rust
impl_effect_node_pin_group!(EffectNodeGrantEffect,
    input => (start => (effect_bundle: Box<dyn EffectBundleTrait>))
    output => (start => (start_effect_entity: Entity), finish => (end_effect_entity: Entity))
);
```

**执行**：`On<EffectNodeExecEvent>` observer（仿 `node/implement/timer.rs:64-115`）：
1. 从 `EffectGraphContext::get_input_value` 取 `EffectValue::BoxReflect(v)`
2. `type_registry.get_type_data::<ReflectEffectBundleTrait>(v.type_id())` → `reflect_trait.get(v.deref())` → `&dyn EffectBundleTrait`
3. `effect_bundle.spawn_scene(&mut commands)`（Q4）→ `node.effects.push(entity)`
4. `executor.start_push_output_pin(OUTPUT_EXEC_START, ...)`

**`EffectBundleTrait`（src/bundle.rs 唯一幸存 trait，Q4）**：

```rust
#[reflect_trait]
pub trait EffectBundleTrait {
    fn build_scene(&self) -> Box<dyn Scene>;
    fn spawn_scene<'a>(&self, commands: &'a mut Commands) -> EntityCommands<'a> {
        commands.spawn_scene(self.build_scene())
    }
}
```

> 注：`build_scene` 返回 `Box<dyn Scene>`——`Scene` 是 `SceneBox: Send + Sync + 'static`，
> `Box<dyn Scene>` 有 `SceneBox` impl（`scene.rs:78` 文档确认）。

**`EffectValue::BoxReflect`（blackboard.rs）**：

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum EffectValue {
    // ...
    #[reflect(ignore)]
    BoxReflect(Box<dyn Reflect>),
}
// 手写 Clone（reflect_clone） + 手写 PartialEq（reflect_partial_eq）
```

> `#[derive(Reflect)]` 对 enum 要求字段 `FromReflect`——`Box<dyn Reflect>` 不满足，故 `#[reflect(ignore)]`；
> 同时 `Clone`/`PartialEq` 手写（`Box<dyn Reflect>` 无 std impl）。`#[reflect(ignore, clone)]` 会因
> `Clone::clone` 编译失败，所以只 `#[reflect(ignore)]` + 手写 impl。

**测试（必须，用户强调）**：grant_effect 是库内零消费者节点 → 必须有集成测试证明链路：

```rust
// tests/grant_effect.rs
#[test]
fn grant_effect_reflect_dispatch_spawns_effect_scene() {
    // 1. 构建 App：register_type 效果 bundle 具体类型 + ReflectEffectBundleTrait
    // 2. 构建图：context + grant_effect 节点 + 输入 pin 值 EffectValue::BoxReflect(Box::new(TestEffectBundle))
    // 3. trigger EffectNodeExecEvent → 断言新实体已生成（带 TestEffectBundle 的组件）
}
```

### 4.7 effect 模块接入编译树（Q5，本次范围）

- `lib.rs` 声明 `pub mod effect`；`EffectPlugin` 加入 `AbilitySubsystemPlugin`。
- **Bundle 迁移**：`EffectStartTagBundle`/`EffectAbortTagBundle`（§4.1 不再删除）→ 组件内联
  （effect 实体由 `spawn_effect` 模板函数或组件组构建）。
- **旧图事件重写**：`EffectNodeStartEvent`/`EffectNodeEvent`/`EffectNodeCheckStartEvent` →
  新架构 `EffectGraphExecEvent`（入口执行口）+ `EffectNodeId` 语义；`EffectGraphMap` 与
  `graph::graph_map::EffectGraphMap` 去重/改名（E2 继承 buff/ability graph_class）。
- **新入口节点**：`EffectNodeEffectEntry`（E1，仿 ability_entry/buff_entry，Plugin 注册 `TypedComponentIds`）。
- **到时销毁**：`time_end_destroy_effect` 触发图 END exec（E3，仿 buff/timer.rs），不直接 despawn。
- **atom_layertag API 对齐**：`container::` → `container_op::`（对照 ability/buff 的 layertag 写法）。
- 依赖矩阵（每个符号 → 现状 → 修复方案）见 explore 报告，随实现 commit 落实。

### 4.8 泛型 Bundle → 组件内联

| 旧 | 新 |
|---|---|
| `graph/bundle.rs::EffectGraphBundle<T>` | `commands.spawn((context, state, tick_state, graph))`（tuple 组件组，无需 Bundle derive） |
| `graph/node/bundle.rs::StateEffectNodeBundle<T>` | `commands.spawn((state_node, execute_state, node_id))` |

### 4.6 其他

- `examples/ability/base_attack.rs` 使用 `StateEffectNodeBundle`/`EffectGraphBundle` → 改 tuple 组件组。
- `impl_effect_node_pin_group!`、`EffectNodeBaseBundle` 引用清理。

## 5. 验收标准

- [ ] 全部 `#[derive(Bundle)]` 删除（`grep -rn "derive(Bundle" crates/atom_ability` 零命中）
- [ ] `BundleTrait`/`AbilityBundleTrait`/`BuffBundleTrait` 删除；`EffectBundleTrait`（spawn_scene 语义）存活且有测试
- [ ] `EffectValue::BoxReflect` 启用（`#[reflect(ignore)]` + 手写 Clone/PartialEq）
- [ ] grant_effect 节点重建于新架构，集成测试通过（反射调度链路真实可用）
- [ ] `graph/base/*` 死代码删除；`InstantEffectNodeBase` 保留迁移
- [ ] **effect 模块接入编译树**（lib.rs `pub mod effect` + `EffectPlugin` 注册 + 旧图事件重写 + layertag API 对齐）
- [ ] `cargo check --workspace` / `clippy -D warnings` / `cargo test` / rustdoc 全绿
- [ ] `cargo run -p atom_ability --example ability --release` 冒烟通过
- [ ] kb/bevy/bsn.md 更新迁移记录（含 BoxReflect 0.19 不支持结论 + `#[reflect(ignore)]` 方案 + effect 接入记录）

## 6. 测试策略（RED → GREEN，test-agent 独立编写）

1. **RED 阶段**：委派 `test-agent` 从 spec 写失败测试：
   - bundle 迁移后 `spawn_ability`/`spawn_buff`/`spawn_ability_owner` 场景产物断言
   - `EffectValue::BoxReflect` Clone/PartialEq 语义
   - grant_effect 反射调度集成测试（核心）
2. **GREEN 阶段**：实现后由 test-agent 独立复验 + spec 偏差报告。
3. 测试命令：`cargo test -p atom_ability --release`。

## 7. 提交拆分（每 commit `ref #7`）

| # | 类型 | 内容 |
|---|---|---|
| 1 | refactor | 删除 `graph/base/*` 死代码 + `graph/bundle.rs` + `node/bundle.rs::StateEffectNodeBundle`；`InstantEffectNodeBase` 移至 `node/mod.rs`；base_attack.rs 改 tuple |
| 2 | feat | `bundle.rs` 重写为 `EffectBundleTrait`（Q4）；`EffectValue::BoxReflect` + 手写 Clone/PartialEq；RED 测试 |
| 3 | feat | grant_effect 重建于新架构（`node/implement/grant_effect.rs`）+ 集成测试 |
| 4 | refactor | ability/buff 侧 `spawn_*` 模板迁移 + example 更新 |
| 5 | feat | **effect 模块接入编译树**（Q5：旧图事件重写 + layertag API 对齐 + `EffectPlugin` 接线 + Bundle 迁移） |
| 6 | docs | kb/bevy/bsn.md 迁移记录 + TENSIONS.md |

## 8. 风险与缓解

| 风险 | 缓解 |
|---|---|
| `Box<dyn Reflect>` 无 Clone/PartialEq → 手写 | `reflect_clone` + `reflect_partial_eq`（blackboard 已有 TryFrom 模式可参考） |
| grant_effect 重建依赖新架构细节 | 以 `node/implement/timer.rs` 为模板（observer + executor + context 三件套） |
| `build_scene` 返回 `Box<dyn Scene>` 生命周期 | `SceneBox` 已是 `Box<dyn Scene>` 惯用路径（`scene.rs:68-78`） |
| bsn! 泛型组件（`spawn_ability_owner::<T>`） | `template_value` + `{expr}` 注入；若泛型受阻，退回 tuple 组件组 |
| effect 接入编译树依赖重（旧事件+旧 layertag API） | 依赖矩阵先行（explore 报告）；按「符号→现状→修复」逐项落地；重写交互而非硬接 |
