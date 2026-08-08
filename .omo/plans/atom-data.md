# Plan: atom_data — bevy_common_assets 驱动的数据访问框架

> **Epic**: [#2](https://github.com/qiboda/atom/issues/2)
> **Branch**: `feat/atom-data`（worktree `.worktrees/atom-data`）
> **状态**: 进行中
> **权威跟踪文档**：本文件（Tasks 表格见文末）

---

## 1. 背景

Luban 生成的二进制 datatables 体系（atom_datatables / atom_cfg / atom_macros / atom_luban_lib）存在痛点：
- 二进制 `.bytes` 格式不可读、不可手工编辑、Luban 生成代码重
- 表访问依赖 `TableReader<T>` SystemParam + 手写 trait 族（MapTable/OneTable/MultiIndexListTable…）
- 跨表引用是字符串外键 + 运行时手动解析，无声明式支持

目标：新建 `atom_data` crate，基于 bevy_common_assets 提供**声明式数据表框架**——`DataAsset` derive（Asset + 反序列化 + 索引构建）+ `DataRegistry` 资源（同步查询）+ `data_ref` 字段级跨表引用。全面替代 datatables 体系。

## 2. 已锁定决策（grill-me shared understanding，见 handoff.md）

| # | 决策 |
|---|---|
| Q1 | **B** 全面替代——atom_ability 数据访问层重写，datatables 系列不再被引用（文件保留原状，不引入 workspace） |
| Q2 | **A** 新建 `atom_data` crate，基于 bevy_common_assets |
| Q3 | **全部格式**支持（json/ron/toml/yaml/csv/msgpack/cbor/xml/postcard），格式由使用方选择，框架不绑定 |
| Q5 | **A** 索引 = 可选 trait + derive，支持单/多重/复合/多值索引 |
| Q6 | **A** `DataAsset` = Asset + 内建索引，`data.get(id)` 直接查 |
| Q7 | **A** 声明式 `data_ref` 字段级跨表引用 |
| Q8 | **A** 依赖 Bevy 原生 `AssetEvent`，查询 lazy |
| Q9 | **默认惰性加载 + 显式 `load`/`unload` 接口** |
| Q10 | **A** `DataRegistry` 资源 + 同步查询；目录约定 `assets/datatables/<表类型名>.json` |

## 3. 技术选型（研究结论，2026-08-08）

| 项 | 结论 |
|---|---|
| 依赖 | `bevy_common_assets 0.17.0`（2026-06-21 发布，依赖 `bevy_app/bevy_asset/bevy_reflect ^0.19.0`，**原生兼容 Bevy 0.19**，无需 fork） |
| 格式 | 9 格式全支持：json/ron/toml/yaml/csv/msgpack/cbor/xml/postcard，**0 默认 feature**，使用方按需开启 |
| 加载器 | bevy_common_assets 提供 `XxxAssetPlugin::<A>::new(&["ext"])` 泛型插件；`app.init_asset::<A>()` + 多格式插件可共存（同一 asset 类型多扩展名路由） |
| 索引挂载点 | `AssetEvent::<T>::LoadedWithDependencies`（本体 + 全部递归依赖就绪）→ 构建索引 |
| AssetLoader 0.19 | trait 签名含 `TypePath` supertrait（0.18 新增）；注册用 `init_asset_loader`（需 FromWorld）或 `register_asset_loader`（实例）；`LoadContext::load_builder()` 取代旧 `NestedLoader`/`load_direct` |
| 查询 lazy | `AssetEvent` 驱动注册，查询时未加载返回 None（Q8/Q9） |
| 目录约定 | `assets/datatables/<表类型名>.json`（Q10） |

**关键决策 D1（设计）：行类型 + 表容器分离**
- `#[derive(DataAsset)]` 用于**行类型**（Bean，如 `AbilityConfig`），只要求 `serde::Deserialize + DataIndexed`
- **表 Asset 是泛型容器** `DataTable<T: DataIndexed>`（atom_data 内定义）：`rows: Vec<T>` + `index: T::Index`（`T::Index` 由宏生成，`HashMap<K, usize>` 行号映射）
- 用户注册 `app.init_asset::<DataTable<AbilityConfig>>()` + `JsonAssetPlugin::<DataTable<AbilityConfig>>::new(&["json"])` 等
- 行类型自身**不是** Asset，无需 Asset/TypePath/VisitAssetDependencies——宏职责最小化，避免与 bevy derive 冲突
- 待验证点（batch 1 spike）：`#[derive(Asset, TypePath)]` 于泛型 `DataTable<T>` 时 `TypePath` 唯一性（不同 T 实例）。若 Bevy 按 TypeId 区分 asset 类型则无碍，按 TypePath 区分则需 fallback 方案（宏生成具名表类型 `{RowName}Table`）——**实现时先查 kb/bevy + /data/codes/Bevy 源码，结论记录到 TENSIONS.md**

**关键决策 D2（设计）：索引系统（Q5）**
```rust
pub trait DataIndexed: Sized + Send + Sync {
    type Index: DataIndex<Self>;      // 宏生成：HashMap 族容器
    // 查询接口由 DataIndex trait 提供，宏生成 impl
}
```
| 索引形态 | 属性写法 | 语义 |
|---|---|---|
| 单键唯一（主） | `#[index(key = "id")]` | `get(&K) -> Option<&T>` |
| 多索引 | 多个 `#[index(...)]` | 每索引一个 map |
| 复合键 | `#[index(key = ("a", "b"))]` | 元组 key `(A, B)` |
| 多值 | `#[index(key = "type", multi)]` | 一 key 多行 `get_all(&K) -> Vec<&T>` |
| 无索引 | 不加 `#[index]` | `DataTable.iter()` 全量迭代 |

**关键决策 D3（设计）：data_ref 跨表引用（batch 2，Q7）**
```rust
#[data_ref(table = "LayerTagConfig", key = "id")]
start_required_layertags: Vec<String>,
```
- 字段**保持原始类型**（`Vec<String>` = 目标表键列表），不侵入序列化格式
- 宏生成解析方法（如 `fn start_required_layertags(&self, data: &DataRegistry) -> Option<Vec<&LayerTagConfig>>`），惰性解析：目标表未加载返回 None（Q8）
- 与现状兼容：现有跨表引用（graph_class / raw_layertag / RevertableLayerTag）全是 String 键 + 运行时惰性解析，data_ref 只是把手动解析声明化

**关键决策 D4（设计）：DataRegistry（batch 2，Q10）**
```rust
pub struct DataRegistry {
    tables: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,  // 按行类型 TypeId 擦除存储
}
impl DataRegistry {
    pub fn get<T: DataIndexed>(&self, key: &T::PrimaryKey) -> Option<&T>;   // 惰性：未加载 None
    pub fn load<T: DataIndexed>(&mut self, server: &AssetServer, path: impl Into<AssetPath>);
    pub fn unload<T: DataIndexed>(&mut self);
    pub fn is_loaded<T: DataIndexed>(&self) -> bool;
}
```
- 系统监听 `AssetEvent<DataTable<T>>::LoadedWithDependencies` → 构建索引 → 存入 registry
- `load` 触发 `asset_server.load`（默认惰性）；`unload` 移除并（可选）释放 handle

---

## 4. Batch 1 — #3: atom_data: DataAsset derive 宏 + 索引系统 + 全格式加载

> Issue: https://github.com/qiboda/atom/issues/3 | Depends: —

### 4.1 目标

建立 `atom_data` crate 骨架：DataAsset derive 宏（行类型 → DataIndexed impl）、索引系统（单/多重/复合/多值/无索引）、DataTable<T> 泛型 Asset、bevy_common_assets 全格式集成、目录约定加载。

### 4.2 任务分解

| # | 任务 | 产出 |
|---|---|---|
| B1-1 | 创建 `crates/atom_data` + `crates/atom_data_macros`（proc-macro），加入 workspace members；依赖 bevy_common_assets 0.17（全 9 格式 feature）+ serde | 两个 crate 骨架，`cargo check` 通过 |
| B1-2 | 实现 `DataIndexed` trait + `DataIndex` trait（get/get_all/iter 查询接口） | trait 定义 + rustdoc |
| B1-3 | 实现 `DataTable<T: DataIndexed>` 泛型 Asset（rows + index + Deserialize 手动 impl：Vec<T> → build index） | DataTable 可反序列化 |
| B1-4 | 实现 `DataAsset` derive 宏：解析 `#[index(...)]` 属性 → 生成 `T::Index` 容器类型 + `DataIndexed`/`DataIndex` impl | 宏生成正确代码 |
| B1-5 | 全格式加载验证：json/ron/toml/yaml/csv/msgpack/cbor/xml/postcard 各一个 `XxxAssetPlugin::<DataTable<T>>` 注册示例 + 加载测试 | 全格式可加载 |
| B1-6 | 目录约定：`DataTable` 加载路径按 `datatables/<表类型名>.json` 约定（示例/测试用） | 约定落地 |
| B1-7 | kb 同步（见 §7） | kb 更新随 commit |

### 4.3 验收标准（#3 子 issue）

- [ ] `#[derive(DataAsset)]` + `#[index(key = "id")]` 生成可编译、可查询的行类型
- [ ] 单键/复合键/多值/多索引/无索引五种形态均有测试覆盖
- [ ] 9 格式全部可加载同一 `DataTable<T>`（格式由使用方注册插件选择，框架不绑定）
- [ ] 新 pub API 全带 `///` rustdoc，`RUSTDOCFLAGS="-Dwarnings" cargo doc --no-deps -p atom_data` 无警告
- [ ] 全门禁通过（§6）

### 4.4 测试设计（RED，test-agent 独立设计）

- 索引构建：`#[index(key = "id")]` → `get(&1)` 命中 / 未命中 None
- 多索引：`#[index(key = "name")]` 独立查询
- 复合键：`(a, b)` 元组查询
- 多值：`multi` 一 key 多行
- 无索引：`iter()` 全量
- 反序列化：JSON 数组 → `DataTable<T>`（id/name 字段正确）
- 全格式：至少 3 种格式（json/ron/toml）加载等价性测试（同数据不同格式 → 相同查询结果）
- 索引重复键：唯一索引遇到重复 key 的行为（预期：error 或 last-wins，实现时定，测试锁定）

---

## 5. Batch 2 — #4: atom_data: DataRegistry 资源 + data_ref 跨表引用

> Issue: https://github.com/qiboda/atom/issues/4 | Depends: #3

### 5.1 目标

DataRegistry 资源（同步查询 + load/unload + AssetEvent 惰性注册）+ `data_ref` 字段级跨表引用（声明式 + 惰性解析）。

### 5.2 任务分解

| # | 任务 | 产出 |
|---|---|---|
| B2-1 | `DataRegistry` 资源 + `DataRegistryPlugin`（AssetEvent::LoadedWithDependencies 监听系统） | registry.get/load/unload/is_loaded 可用 |
| B2-2 | `data_ref` 属性宏支持：解析 `#[data_ref(table = "...", key = "...")]` → 生成惰性解析方法 | 字段级跨表引用可用 |
| B2-3 | 跨表引用集成测试：Ability → LayerTagConfig 两表，load 顺序无关（先/后加载均能解析） | 顺序无关验证 |
| B2-4 | 未加载查询返回 None + unload 后失效 | 惰性语义锁定 |
| B2-5 | kb 同步（见 §7） | kb 更新随 commit |

### 5.3 验收标准（#4 子 issue）

- [ ] `data.get::<T>(id)` 惰性：未加载 None，加载后可查
- [ ] `data.load::<T>(path)` / `data.unload::<T>()` 显式生命周期控制（Q9）
- [ ] `#[data_ref(table = "X", key = "id")]` 字段生成解析方法，目标表未加载返回 None
- [ ] 全门禁通过（§6）

### 5.4 测试设计（RED，test-agent 独立设计）

- 单表 get/load/unload/is_loaded 生命周期
- 两表跨表引用：引用方先加载、被引用方后加载 → 解析仍成功（惰性解析时机）
- data_ref 键不存在 → 跳过/None（行为实现时定，测试锁定）
- 重复 load 幂等性

---

## 6. Batch 3 — #5: atom_ability: 数据访问层迁移到 atom_data

> Issue: https://github.com/qiboda/atom/issues/5 | Depends: #3, #4

### 6.1 目标

atom_ability 全面迁移到 atom_data，datatables 系列不再被引用。含 manifest/workspace 修复（前置）。

### 6.2 任务分解

| # | 任务 | 产出 |
|---|---|---|
| B3-0 | **manifest 校验（前置）**：已随 rebase 同步 main（`4e87f59` workspace 迁移 11 crates 完成，workspace.dependencies 已补齐 once_cell/smallvec/thiserror/dotenv/paste/serde——atom_ability 在 main 上可构建）。仅需确认 `cargo check --workspace` 全绿 + 移除 atom_datatables 依赖后构建仍通过 | `cargo check --workspace` 全绿 |
| B3-1 | 用 serde 结构体 + `#[derive(DataAsset)]` 定义原子表类型：`AbilityConfig` / `BuffConfig` / `LayerTagConfig`（字段与现有 Bean 一致：id/name/desc/graph_class/cd/activation_type/start_required_layertags…；layertag 用 `#[data_ref]`） | 3 个数据表类型 |
| B3-2 | 替换 `TableReader<T>` SystemParam → `DataRegistry` 查询（stateset/mod.rs、buff/event.rs、examples/ability/main.rs 三处） | TableReader 零引用 |
| B3-3 | 删除 `TbAbilityRow`/`TbBuffRow`（key+data 分离组件）→ 数据直接 + 索引（handoff 锁定）；`AbilityBundle`/`BuffBundle` 改为携带行数据/键，observer 经 registry 查数据 | Row 组件删除 |
| B3-4 | 跨表引用声明化：`start_required_layertags: Vec<String>` / `graph_class` 等改用 `#[data_ref]`（graph_class 指向 EffectGraphBuilderMap 的按名解析保留现状或声明化——实现时评估） | data_ref 落地 |
| B3-5 | `trigger_buff_add_event`（TableReader<TbBuff> 死代码，全仓无引用）——**删除**，记录到 issue 注释 | 死代码清除 |
| B3-6 | 迁移示例 `examples/ability/main.rs` 到 atom_data 加载流程；assets 数据文件 → JSON 格式 | 示例可运行 |
| B3-7 | 删除 atom_ability 对 atom_datatables 的 Cargo 依赖；kb/AGENTS.md 同步（见 §7） | datatables 零引用 |
| B3-8 | 全量验证：`cargo run -p atom_ability --example ability --release` 冒烟 + 全门禁 | 冒烟证据 |

### 6.3 验收标准（#5 子 issue）

- [ ] atom_ability 无任何 `atom_datatables`/`atom_luban_lib`/`TbAbilityRow`/`TableReader` 引用
- [ ] 数据访问全部经 `DataRegistry` + `DataAsset` 声明
- [ ] `cargo check --workspace` + clippy + test + doc 全绿
- [ ] 示例 `ability` `--release` 实跑冒烟（渲染终态证据）

### 6.4 测试设计（RED，test-agent 独立设计）

- AbilityConfig/BuffConfig/LayerTagConfig 反序列化 + 索引查询
- Bundle 构造：能力/增益 spawn 后数据正确（graph_class、layertag 解析）
- layertag 跨表引用解析（Ability 引用 LayerTagConfig）
- 回归：现有 ability 行为测试（若有）迁移到新数据层

---

## 7. kb 同步（文档映射）

| 变更 | kb 文件 | 内容 |
|---|---|---|
| bevy_common_assets 0.17 兼容 Bevy 0.19 | `kb/bevy/migration-index.md` | 新增条目：0.19 资产加载可用 bevy_common_assets 0.17.0；AssetLoader TypePath supertrait；`load_builder()` 取代 `NestedLoader`/`load_direct` |
| 数据访问架构变更（ADR） | `kb/ARCHITECTURE.md` | 新增 ADR：`atom_data` 框架替代 datatables（what + why + why-not：bevy_common_assets 全格式 vs Luban 二进制） |
| 工具链摩擦 | `kb/TENSIONS.md` | 迁移发现：atom_ability 当前无法构建（workspace deps 缺失）；DataTable<T> TypePath 泛型唯一性结论（spike 后记录） |
| 项目状态 | `AGENTS.md` | workspace 当前状态更新（atom_data 加入；atom_ability/layertag 迁回）；crate 表加 atom_data |
| 游戏系统 | `kb/project/game/README.md` | 技能数据加载方式变更摘要（如该文件已有内容） |

## 8. 验证门禁（每 batch 完成时全跑）

```sh
cargo check --workspace
cargo clippy --workspace -- -A dead_code -D warnings
cargo test --workspace        # nextest 可用则 cargo nextest run --workspace
RUSTDOCFLAGS="-Dwarnings" cargo doc --no-deps -p atom_data   # 新 pub 项时
```

- Batch 3 额外：`cargo run -p atom_ability --example ability --release`（超时 30s，冒烟要有渲染终态证据）
- 运行/测试必须 `--release`（Bevy debug 极慢）；clippy 上游 bevy_ui_widgets 警告不算失败

## 9. 风险与待验证点

| 风险 | 影响 | 缓解 |
|---|---|---|
| `DataTable<T>` 泛型 TypePath 唯一性 | 高——若 Bevy 按 TypePath 区分 asset 类型，不同 T 冲突 | batch 1 spike 先验证（查 kb + /data/codes/Bevy 源码）；fallback：宏生成具名表类型 |
| csv 格式多行表反序列化 | 中——csv 是记录流，`Vec<T>` 反序列化语义需确认 | spike 验证 bevy_common_assets 0.17 csv loader 行为 |
| 复合键宏语法 | 低——`#[index(key = ("a","b"))]` 解析 | 宏解析用 syn 完整支持 tuple 字面量 |
| bevy 子 crate patch 覆盖 | 中——第三方库直接依赖 bevy_app/bevy_asset 等子 crate 时 crates.io 版本缺 cfg_select | 已解决：patch 增补 bevy_app/bevy_asset/bevy_reflect（记录于 TENSIONS 2026-08-08） |
| workspace members 合并 | 低——main 迁移 11 crates 与本分支新增 atom_data 的合并 | 已解决：rebase + 手动合并 Cargo.toml（12 members） |

---

## 10. Tasks 跟踪表

### Batch 1 (#3)
| Status | Issue | Task | Depends On |
|--------|-------|------|------------|
| done | #3 | B1-1 atom_data + macros crate 骨架入 workspace | — |
| done | #3 | B1-2 DataIndexed/DataIndex trait | B1-1 |
| done | #3 | B1-3 DataTable<T> 泛型 Asset | B1-2 |
| done | #3 | B1-4 DataAsset derive 宏（index 属性解析） | B1-2 |
| done | #3 | B1-5 全格式加载验证（9 格式） | B1-3, B1-4 |
| done | #3 | B1-6 目录约定 datatables/<表类型名>.json | B1-3 |
| done | #3 | B1-7 kb 同步 | B1-5 |

### Batch 2 (#4)
| Status | Issue | Task | Depends On |
|--------|-------|------|------------|
| done | #4 | B2-1 DataRegistry + Plugin（AssetEvent 监听） | #3 |
| done | #4 | B2-2 data_ref 属性宏 | #3 |
| done | #4 | B2-3 跨表引用集成测试（顺序无关） | B2-1, B2-2 |
| done | #4 | B2-4 惰性语义（None/unload）测试 | B2-1 |
| done | #4 | B2-5 kb 同步 | B2-4 |

### Batch 3 (#5)
| Status | Issue | Task | Depends On |
|--------|-------|------|------------|
| pending | #5 | B3-0 manifest/workspace 修复 | #4 |
| pending | #5 | B3-1 原子表类型 serde + DataAsset 定义 | #4 |
| pending | #5 | B3-2 TableReader → DataRegistry 替换 | B3-0, B3-1 |
| pending | #5 | B3-3 TbAbilityRow/TbBuffRow 删除 + Bundle 重构 | B3-2 |
| pending | #5 | B3-4 data_ref 声明化跨表引用 | B3-3 |
| pending | #5 | B3-5 trigger_buff_add_event 死代码删除 | B3-3 |
| pending | #5 | B3-6 示例迁移 + JSON 资产 | B3-4 |
| pending | #5 | B3-7 datatables 依赖移除 + kb/AGENTS 同步 | B3-5 |
| pending | #5 | B3-8 全门禁 + 冒烟 | B3-6, B3-7 |
