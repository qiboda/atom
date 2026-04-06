use bevy::asset::Asset;

pub trait Table: Asset {
    type Value;
}

pub trait OneTable: Table {
    fn get_data(&self) -> Self::Value;
}

pub trait ListTable: Table {}

pub trait MapTable: Table {
    type Key;
    type List;
    type Map;

    fn get_row(&self, key: &Self::Key) -> Option<Self::Value>;

    fn get_data_list(&self) -> &Self::List;

    fn get_data_map(&self) -> &Self::Map;
}

pub trait MultiUnionIndexListTable: ListTable {
    type Key;
    type List;
    type Map;

    fn get_row_by_key(&self, key: &Self::Key) -> Option<Self::Value>;

    fn get_data_list(&self) -> &Self::List;

    fn get_data_map(&self) -> &Self::Map;
}

pub trait MultiIndexListTable<'a>: ListTable {
    type Key;
    type List;
    type Map;

    fn get_row_by(&self, key: &Self::Key) -> Option<Self::Value>;

    fn get_data_list(&self) -> &Self::List;

    fn get_data_map_by(&'a self, key: &Self::Key) -> Self::Map;
}

pub trait NotIndexListTable: ListTable {
    type List;

    fn iter(&self) -> impl Iterator<Item = &Self::Value>;

    fn get_data_list(&self) -> &Self::List;
}
