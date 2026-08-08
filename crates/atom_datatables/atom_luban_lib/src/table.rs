use bevy::asset::Asset;

/// 数据表基础 trait：一张表是一个 Bevy [`Asset`]。
pub trait Table: Asset {
    /// 单行数据的值类型。
    type Value;
}

/// 单表：只含一行数据。
pub trait OneTable: Table {
    /// 返回表中的数据。
    fn get_data(&self) -> Self::Value;
}

/// 列表表：无索引，按顺序存储多行数据。
pub trait ListTable: Table {}

/// 映射表：按键索引多行数据。
pub trait MapTable: Table {
    /// 行的键类型。
    type Key;
    /// 行的列表类型。
    type List;
    /// 行的映射类型。
    type Map;

    /// 按键查找一行数据；不存在时返回 `None`。
    fn get_row(&self, key: &Self::Key) -> Option<Self::Value>;

    /// 返回全部行组成的列表。
    fn get_data_list(&self) -> &Self::List;

    /// 返回全部行组成的映射。
    fn get_data_map(&self) -> &Self::Map;
}

/// 多联合索引列表表：支持按多个键中的任意一个查找行。
pub trait MultiUnionIndexListTable: ListTable {
    /// 行的键类型。
    type Key;
    /// 行的列表类型。
    type List;
    /// 行的映射类型。
    type Map;

    /// 按键查找一行数据；不存在时返回 `None`。
    fn get_row_by_key(&self, key: &Self::Key) -> Option<Self::Value>;

    /// 返回全部行组成的列表。
    fn get_data_list(&self) -> &Self::List;

    /// 返回全部行组成的映射。
    fn get_data_map(&self) -> &Self::Map;
}

/// 多索引列表表：按主键查询，并可按键返回一个子映射。
pub trait MultiIndexListTable<'a>: ListTable {
    /// 行的键类型。
    type Key;
    /// 行的列表类型。
    type List;
    /// 行的映射类型。
    type Map;

    /// 按键查找一行数据；不存在时返回 `None`。
    fn get_row_by(&self, key: &Self::Key) -> Option<Self::Value>;

    /// 返回全部行组成的列表。
    fn get_data_list(&self) -> &Self::List;

    /// 按键返回对应的子映射。
    fn get_data_map_by(&'a self, key: &Self::Key) -> Self::Map;
}

/// 无索引列表表：按顺序迭代全部行。
pub trait NotIndexListTable: ListTable {
    /// 行的列表类型。
    type List;

    /// 迭代全部行。
    fn iter(&self) -> impl Iterator<Item = &Self::Value>;

    /// 返回全部行组成的列表。
    fn get_data_list(&self) -> &Self::List;
}
