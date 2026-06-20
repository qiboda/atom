# BSN (Bevy Scene Notation)

> Bevy 0.19 的新一代场景系统。`bsn!` 宏 + `.bsn` 资产文件，声明式组合 ECS 实体/组件/关系。

## 是什么

BSN 是 Bevy 的 **entity spawning DSL**，替代旧的 `Bundle` + `commands.spawn()` 模式。
同组概念里：

| 概念 | 用途 |
|------|------|
| `Bundle` | 老方案，组件集合 trait |
| `bsn!` 宏 | **新方案**，Rust 内联场景声明 |
| `.bsn` 文件 | 资产场景（0.19 只支持代码内联，文件 loader 等 0.20） |
| `bevy_scene` | 新 crate（场景系统） |
| `bevy_world_serialization` | 老 `bevy_scene` 改名（保留至 GLTF→BSN 迁移完成） |

## 基本语法

### 单实体

```rust
bsn! {
    Player {
        score: 0,
    }
}
```

等效于 `commands.spawn(PlayerBundle { score: 0, ..default() })`，但：
- 无需 `Bundle` trait 实现
- 无需 `Default`（未设置字段自动 default）
- 无字段 unit 组件直接写类型名：

```rust
bsn! { Player }         // = commands.spawn(Player)
```

### 多实体 + 关系

```rust
bsn! {
    Player
    Children [
        Sword,
        Shield,
    ]
    Inventory [
        Apple { count: 3 },
        Potion,
    ]
}
```

`Children[v]` 是 ECS 父子关系；`Inventory[v]` 是自定义关系（1:N）。

### 命名引用

```rust
bsn! {
    #root
    Children [
        (#button0 Button)
        (#button1 Button)
        :"shared_popup.bsn"
    ]
}
```

`#name` 给实体命名，可在同场景内用 `Reference(#name)` 引用，也可跨场景。

### 模板函数

```rust
fn button(label: &str) -> impl Scene {
    bsn! {
        Button
        Node { width: px(150), height: px(65) }
        Children [ Text(label) ]
    }
}

bsn! {
    (button("Ok") on(|e: On<Pointer<Press>>| println!("Ok")))
    (button("Cancel") BackgroundColor(Color::srgb(0.4, 0.15, 0.15)))
}
```

模板是返回 `impl Scene` 的函数。调用时：
- `(f())` 括号注入——允许追加组件、observer、children
- `Reference(f())` 引用注入——模板的 named entity 可被外部引用

### 外链资产

```rust
bsn! {
    :"scene.gltf#Scene0"
    Transform { translation: Vec3 { x: 10. } }
}
```

`:"path"` 加载外部场景（`.gltf` / `.bsn` / `.glb`），`#name` 指定场景内的具名实体。

### SceneList（多实体列表）

```rust
fn scene() -> impl SceneList {
    bsn_list![Camera2d, ui()]
}
```

`bsn_list!` 生成多个顶层实体（非父子）。

## 核心约束

### Component 不会触发 Bundle 宏

BSN 里的 `ComponentName { field: value }` 直接写 component，不经过 `Bundle` 的 derive。这意味着：
- **不会**触发 `#[require]` 元属性（不自动添加必需组件）
- **不会**展开复杂的 bundle 嵌套

### 需要 Default

BSN 用 `..Default::default()` 填充未设字段 → component 必须 derive `Default`。

### Handle 自动转换

```rust
bsn! { TextFont { font: "fonts/FiraSans-Bold.ttf" } }
// "path" 自动转为 HandleTemplate::Path → 场景初始化时加载 asset
```

## BSN vs 旧方案

| | `commands.spawn(Bundle)` | `bsn!` |
|---|---|---|
| 粒度 | 以 Bundle 为单位 | 以 Component 为单位 |
| 未设字段 | Bundle::default() | Component::default() |
| 关系 | 手动 `insert_related` | 内联 `Children[x]` `Relation[x]` |
| 命名 | 无 | `#name` + `Reference(#name)` |
| 资产引用 | `AssetServer::load` | `:"path.bsn"` 内联 |
| 模板 | 函数返回 Bundle | 函数返回 `impl Scene` |
| Observer | `commands.observe(...)` | `on(|...| { ... })` 内联 |

## 当前状态 (0.19)

**已有**:
- `bsn!` 宏（内联场景）
- `bsn_list!` 宏
- 实体命名 `#name` + `Reference(#name)`
- 关系 `Children[x]` `Relation[x]`
- 模板函数 `impl Scene`
- Observer 内联 `on(|e| ...)`
- Feathers 控件已全面迁移到 BSN

**未完成**:
- `.bsn` 文件 asset loader（等 0.20）
- GLTF→BSN 自动转换（等 0.20）
- World→BSN 序列化（等 0.20）

## 迁移 (old → new)

```
bevy::scene  →  bevy::world_serialization（老 crate，临时）
bevy_scene   →  bevy::scene（新 crate，BSN）
```

## 项目适用性

`atom_terrain` 目前使用 `atom_pqef` 等 crate 的 Bundle + spawn 模式。后续对 UI 或复杂实体场景可考虑 BSN。

## 参考

- `/data/codes/Bevy/examples/scene/bsn.rs` — 基础示例
- `/data/codes/Bevy/release-content/release-notes/` — 0.19 release notes
- `/data/codes/Bevy/crates/bevy_scene/` — 源码
- `https://docs.rs/bevy/0.19.0/bevy/scene/macros/macro.bsn.html`
