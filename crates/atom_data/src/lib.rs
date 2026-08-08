//! atom_data — 声明式数据表框架（bevy_common_assets 驱动，全面替代 Luban 二进制 datatables 体系）。
//!
//! 权威 spec：`.omo/plans/atom-data.md`（issue #3 Batch 1）。
//! 关键决策：
//! - **D1**（设计）：行类型 + 表容器分离——`#[derive(DataAsset)]` 用于**行类型**（Bean），
//!   只要求 `serde::Deserialize + DataIndexed`；表 Asset 是泛型容器 [`DataTable<T>`]
//!   （`rows: Vec<T>` + `T::Index` 行号映射）。
//! - **D2**（设计）：索引系统——`DataIndexed`（声明索引容器类型）+ `DataIndex`（构建契约），
//!   `#[index(...)]` 属性形态：单键唯一（主/次）、复合键 `("a","b")`、多值 `multi`、无索引。
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

use core::fmt;

pub use atom_data_macros::DataAsset;
pub use bevy::prelude::Asset;
pub use bevy::prelude::TypePath;

/// 行类型索引契约：声明行的索引容器类型（由 `#[derive(DataAsset)]` 宏生成 impl）。
///
/// `TypePath` supertrait：`DataTable<T>: Asset` 的 `TypePath` derive 会给 `T` 加 `TypePath` bound，
/// 行类型必须实现 `TypePath`——由宏代生成。
pub trait DataIndexed: Sized + Send + Sync + TypePath {
    /// 该行类型的索引容器（宏生成：`HashMap<K, usize>` 族，multi 为 `HashMap<K, Vec<usize>>`）。
    type Index: DataIndex<Self>;
}

/// 索引容器构建契约。
///
/// `'static` supertrait：`DataTable<T>: Asset` 需要 `T::Index: 'static`（Asset 的 `'static` supertrait）。
pub trait DataIndex<T>: Default + Send + Sync + 'static {
    /// 从全部行构建索引。
    ///
    /// 唯一索引（非 multi）遇到重复 key 返回 `Err`（D2 锁定 error 分支，拒绝静默 last-wins）；
    /// 多值索引天然允许多行共享 key，不报错。
    fn build(rows: &[T]) -> Result<Self, String>;
}

/// 泛型数据表 Asset：`rows: Vec<T>` + 内建索引（D1）。
///
/// 两条构建路径：`DataTable::from_rows`（纯逻辑）与 `Deserialize`（反序列化后自动构建索引）。
/// 泛型 `TypePath` 唯一性已 spike 验证：Bevy 的 `TypePath` derive 对泛型类型使用
/// `GenericTypePathCell`（按 `TypeId` 缓存），不同 `T` 实例得到不同 type path，无冲突。
#[derive(Asset, TypePath)]
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
