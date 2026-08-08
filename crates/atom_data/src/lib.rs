//! atom_data — 声明式数据表框架（bevy_common_assets 驱动，全面替代 Luban 二进制 datatables 体系）。
//!
//! 权威 spec：`.omo/plans/atom-data.md`（issue #3 Batch 1 + issue #4 Batch 2）。
//! 关键决策：
//! - **D1**（设计）：行类型 + 表容器分离——`#[derive(DataAsset)]` 用于**行类型**（Bean），
//!   只要求 `serde::Deserialize + DataIndexed`；表 Asset 是泛型容器 [`DataTable<T>`]
//!   （`rows: Vec<T>` + `T::Index` 行号映射）。
//! - **D2**（设计）：索引系统——`DataIndexed`（声明索引容器类型）+ `DataIndex`（构建契约），
//!   `#[index(...)]` 属性形态：单键唯一（主/次）、复合键 `("a","b")`、多值 `multi`、无索引。
//! - **D3**（设计，Batch 2）：`#[data_ref(table = "...", key = "...")]` 字段级跨表引用——
//!   字段保持原始类型，宏生成 `resolve_{field}` 惰性解析方法。
//! - **D4**（设计，Batch 2）：[`DataRegistry`] 资源——按行类型 `TypeId` 擦除存储，
//!   同步查询 `get::<T>(&pk)` 惰性（未加载 None）。
//! - Q3：全部格式支持（json/ron/toml/yaml/csv/msgpack/cbor/xml/postcard），格式由使用方
//!   bevy_common_assets 插件选择，框架不绑定。
//!
//! # 使用示例
//!
//! ```ignore
//! #[derive(serde::Deserialize, atom_data::DataAsset)]
//! #[index(key = "id")]
//! #[index(key = "name")]
//! #[index(key = ("a", "b"))]
//! #[index(key = "type", multi)]
//! struct AbilityConfig {
//!     id: i32,
//!     name: String,
//!     a: i32,
//!     b: i32,
//!     r#type: i32,
//! }
//!
//! let table: DataTable<AbilityConfig> = DataTable::from_rows(vec![/* ... */])?;
//! table.get(&1);            // 主索引
//! table.get_by_name(&s);    // 次索引
//! table.get_by_pair(&(1, 2)); // 复合键
//! table.get_all_by_type(&1);  // 多值索引
//! ```

#![deny(missing_docs)]

use bevy::asset::{AssetEvent, AssetPath, AssetServer, Assets, Handle};
use bevy::prelude::{App, MessageReader, Plugin, Res, ResMut, Update};
use core::fmt;
use core::hash::Hash;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

pub use atom_data_macros::DataAsset;
pub use bevy::prelude::Asset;
pub use bevy::prelude::Resource;
pub use bevy::prelude::TypePath;

/// 行类型索引契约：声明行的索引容器类型（由 `#[derive(DataAsset)]` 宏生成 impl）。
///
/// `TypePath` supertrait：`DataTable<T>: Asset` 的 `TypePath` derive 会给 `T` 加 `TypePath` bound，
/// 行类型必须实现 `TypePath`——由宏代生成。
/// `'static` supertrait：`DataRegistry` 经 `TypeId` 擦除存储需要行类型 `'static`。
pub trait DataIndexed: Sized + Send + Sync + 'static + TypePath {
    /// 该行类型的索引容器（宏生成：`HashMap<K, usize>` 族，multi 为 `HashMap<K, Vec<usize>>`）。
    type Index: DataIndex<Self>;

    /// 主索引键类型（无主索引行类型 → `()`，Batch 2）。
    type PrimaryKey: Hash + Eq + Clone;

    /// 提取主索引键值（无主索引 → `()`）。
    fn primary_key(&self) -> Self::PrimaryKey;
}

/// 索引容器构建 + 查询契约。
///
/// `'static` supertrait：`DataTable<T>: Asset` 需要 `T::Index: 'static`（Asset 的 `'static` supertrait）。
/// `Clone` supertrait：`DataTable<T>` 的 `Clone` derive（B2-1 集成层）要求 `T::Index: Clone`——
/// 由宏生成的索引容器 `#[derive(Clone)]` 满足（无索引行类型 `()` 亦满足）。
pub trait DataIndex<T>: Default + Send + Sync + Clone + 'static
where
    T: DataIndexed,
{
    /// 从全部行构建索引。
    ///
    /// 唯一索引（非 multi）遇到重复 key 返回 `Err`（D2 锁定 error 分支，拒绝静默 last-wins）；
    /// 多值索引天然允许多行共享 key，不报错。
    fn build(rows: &[T]) -> Result<Self, String>;

    /// 主索引查询。
    ///
    /// 输出生命周期显式绑定 `rows` 参数（不能靠 elision 绑 `&self`——[`DataRegistry`] 经
    /// `&self.index` 调用时会 borrow-check 失败，Batch 2 实证）。
    fn get<'a>(&self, rows: &'a [T], key: &T::PrimaryKey) -> Option<&'a T>;
}

/// 泛型数据表 Asset：`rows: Vec<T>` + 内建索引（D1）。
///
/// 两条构建路径：`DataTable::from_rows`（纯逻辑）与 `Deserialize`（反序列化后自动构建索引）。
/// 泛型 `TypePath` 唯一性已 spike 验证：Bevy 的 `TypePath` derive 对泛型类型使用
/// `GenericTypePathCell`（按 `TypeId` 缓存），不同 `T` 实例得到不同 type path，无冲突。
/// `Clone`：`DataRegistry` 的 sync 系统在 `LoadedWithDependencies` 后 clone 进注册表
/// （B2-1 集成层要求，`T::Index` 由宏生成 `#[derive(Clone)]`）。
#[derive(Asset, TypePath, Clone)]
pub struct DataTable<T: DataIndexed> {
    rows: Vec<T>,
    index: T::Index,
}

impl<T: DataIndexed> DataTable<T> {
    /// 从行序列构建数据表并构建索引（唯一索引重复键报错）。
    pub fn from_rows(rows: Vec<T>) -> Result<Self, String> {
        let index = T::Index::build(&rows)?;
        Ok(Self { rows, index })
    }

    /// 按行插入顺序迭代全部行。
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.rows.iter()
    }

    /// 全部行切片（插入顺序）。
    pub fn rows(&self) -> &[T] {
        &self.rows
    }

    /// 索引容器引用。
    pub fn index(&self) -> &T::Index {
        &self.index
    }

    /// 通用主索引查询（[`DataRegistry`] 经此访问；与宏生成的 `{T}Queries::get` 不冲突——名字不同）。
    pub fn get_primary(&self, key: &T::PrimaryKey) -> Option<&T> {
        self.index.get(&self.rows, key)
    }

    /// 行数。
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// 是否为空表。
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// `Debug` 只要求 `T::Index: Debug`（行类型无需 `Debug`）。
impl<T: DataIndexed> fmt::Debug for DataTable<T>
where
    T::Index: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DataTable")
            .field("len", &self.rows.len())
            .field("index", &self.index)
            .finish()
    }
}

/// 反序列化（双形态，B1-3）：
/// - **顶层序列**（JSON / RON 数组）→ 行序列
/// - **单键 `rows` map**（TOML：根必须是 table）→ `rows` 键的值取行序列
///
/// 反序列化后自动构建索引（`DataTable::from_rows`）；索引构建错误（如重复唯一键）传播为反序列化错误。
impl<'de, T> serde::Deserialize<'de> for DataTable<T>
where
    T: serde::Deserialize<'de> + DataIndexed,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct TableVisitor<T>(core::marker::PhantomData<T>);

        impl<'de, T> serde::de::Visitor<'de> for TableVisitor<T>
        where
            T: serde::Deserialize<'de> + DataIndexed,
        {
            type Value = DataTable<T>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a sequence of rows, or a map with a `rows` key")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut rows = Vec::new();
                while let Some(row) = seq.next_element::<T>()? {
                    rows.push(row);
                }
                DataTable::from_rows(rows).map_err(serde::de::Error::custom)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut rows: Option<Vec<T>> = None;
                while let Some(key) = map.next_key::<String>()? {
                    if key == "rows" {
                        rows = Some(map.next_value()?);
                    } else {
                        map.next_value::<serde::de::IgnoredAny>()?;
                    }
                }
                let rows = rows.ok_or_else(|| serde::de::Error::missing_field("rows"))?;
                DataTable::from_rows(rows).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_any(TableVisitor(core::marker::PhantomData))
    }
}

/// data_ref 键转换契约：引用列表的元素（`String`）转换为主键类型。
///
/// 仅在 `#[data_ref(table = "...", key = "...")]` 生成的 `resolve_{field}` 方法被使用时要求——
/// 目标表必须实现此 trait（宏生成代码的 where 约束把编译错误指向这里，比 `FromStr` 可读）。
/// 常见主键类型（i32/u32/i64/String 等）经 blanket impl 自动满足；解析失败返回 `None`
/// → 该键被跳过（与 `data_ref` 测试契约的 skip 语义一致）。
pub trait DataRefKey: Sized {
    /// 从引用字符串解析主键（解析失败返回 `None` → 该键被跳过）。
    fn from_ref_str(s: &str) -> Option<Self>;
}

/// `FromStr` 键类型的 blanket 实现（i32/u32/i64/String 等自动满足）。
impl<K: core::str::FromStr> DataRefKey for K {
    fn from_ref_str(s: &str) -> Option<Self> {
        s.parse::<K>().ok()
    }
}

/// 按行类型 `TypeId` 擦除存储的表注册表（D4/Q10）。
///
/// 同步查询入口：`data.get::<T>(&pk)` 惰性查询（未加载返回 None）；`insert`/`unload` 显式
/// 生命周期管理（Q9）；`load`/`reload` 走 AssetServer + AssetEvent 集成层（B2-1）——
/// 表以 `Arc<dyn Any + Send + Sync>` 擦除存储，按行类型 `TypeId` 隔离。
#[derive(Default, Resource)]
pub struct DataRegistry {
    tables: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
    /// `load` 记录的行类型 → 资产路径（`reload` 复用；`TypeId` 与 `tables` 键同域）。
    paths: HashMap<TypeId, AssetPath<'static>>,
}

impl DataRegistry {
    /// 注册/替换表（同类型重复 insert = 替换，registry 级幂等——重复 load 不累积表）。
    pub fn insert<T: DataIndexed>(&mut self, table: DataTable<T>) {
        self.tables.insert(TypeId::of::<T>(), Arc::new(table));
    }

    /// 惰性查询：目标表未加载 → None；已加载 → 按主索引查询（Q8）。
    pub fn get<T: DataIndexed>(&self, key: &T::PrimaryKey) -> Option<&T> {
        self.table::<T>()?.get_primary(key)
    }

    /// 表容器引用（data_ref 解析用）。
    pub fn table<T: DataIndexed>(&self) -> Option<&DataTable<T>> {
        self.tables
            .get(&TypeId::of::<T>())?
            .downcast_ref::<DataTable<T>>()
    }

    /// 是否已加载该行类型的表。
    pub fn is_loaded<T: DataIndexed>(&self) -> bool {
        self.tables.contains_key(&TypeId::of::<T>())
    }

    /// 显式释放（Q9）；unload 后 `get` 返回 None。保留记录的路径——`reload` 可复用。
    pub fn unload<T: DataIndexed>(&mut self) {
        self.tables.remove(&TypeId::of::<T>());
    }

    /// 显式加载（Q9/B2-1）：触发 AssetServer 加载并记录路径。数据经 sync 系统在
    /// `AssetEvent::LoadedWithDependencies` 后进入 registry（惰性注册，Q8）。
    ///
    /// `T: DeserializeOwned`：加载器（如 bevy_common_assets）要求整表可反序列化，
    /// 行类型须 derive `serde::Deserialize`。
    pub fn load<T: DataIndexed + serde::de::DeserializeOwned>(
        &mut self,
        server: &AssetServer,
        path: impl Into<AssetPath<'static>>,
    ) -> Handle<DataTable<T>> {
        let path = path.into();
        let handle = server.load::<DataTable<T>>(path.clone());
        self.paths.insert(TypeId::of::<T>(), path);
        handle
    }

    /// 重新加载（issue #4 验收）：用 `load` 记录的路径重新触发加载。
    ///
    /// 未记录过路径 → 返回 `None`。已加载资产（handle 存活）走 `AssetServer::reload`
    /// 强制从磁盘重读——普通 `load` 对缓存资产直接返回，不会重发
    /// `LoadedWithDependencies`；未加载（如上次失败）由 `load` 新起加载。两条路径
    /// 收敛于同一 sync 系统重新入注册表。
    pub fn reload<T: DataIndexed + serde::de::DeserializeOwned>(
        &mut self,
        server: &AssetServer,
    ) -> Option<Handle<DataTable<T>>> {
        let path = self.paths.get(&TypeId::of::<T>())?.clone();
        let handle = server.load::<DataTable<T>>(path.clone());
        server.reload(path);
        Some(handle)
    }
}

/// 注册 DataRegistry 资源；`register_table::<T>` 为每个行类型注册加载同步系统（B2-1）。
pub struct DataRegistryPlugin;

impl Plugin for DataRegistryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DataRegistry>();
    }
}

impl DataRegistryPlugin {
    /// 注册某行类型的加载同步系统（每种行类型调用一次）。
    ///
    /// `T: DeserializeOwned`：与 `load`/`reload` 的加载契约对齐（可经 AssetServer 加载
    /// 的行类型才能被 sync 入注册表）。
    pub fn register_table<T>(app: &mut App)
    where
        T: DataIndexed + Clone + serde::de::DeserializeOwned + 'static,
    {
        app.add_systems(Update, sync_table::<T>);
    }
}

/// 每行类型一个的加载同步系统：监听 `AssetEvent::<DataTable<T>>::LoadedWithDependencies`
/// （本体 + 全部依赖就绪，Bevy 0.19 message API），将加载的表 clone 进 DataRegistry
/// （惰性注册，Q8）。重复事件幂等——registry 同类型 insert = 替换。
pub fn sync_table<T: DataIndexed + Clone + 'static>(
    mut registry: ResMut<DataRegistry>,
    assets: Res<Assets<DataTable<T>>>,
    mut events: MessageReader<AssetEvent<DataTable<T>>>,
) {
    for ev in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = ev {
            if let Some(table) = assets.get(*id) {
                registry.insert(table.clone());
            }
        }
    }
}
