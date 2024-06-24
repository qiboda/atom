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

    fn get_row(&self, key: &Self::Key) -> Option<Self::Value>;
}

pub trait MultiUnionIndexListTable: ListTable {
    type Key;

    fn get_row_by_key(&self, key: &Self::Key) -> Option<Self::Value>;
}

pub trait MultiIndexListTable: ListTable {
    type Key;

    fn get_row_by(&self, key: &Self::Key) -> Option<Self::Value>;
}

pub trait NotIndexListTable: ListTable {
    fn iter(&self) -> impl Iterator<Item = &Self::Value>;
}
