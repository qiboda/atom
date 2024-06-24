use bevy::asset::Assets;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use cfg::Tables;
use luban_lib::table::*;

#[derive(SystemParam)]
pub struct TableReader<'w, Tb: Table> {
    pub tables: Res<'w, Tables>,
    pub table: Res<'w, Assets<Tb>>,
}

impl<Tb: MapTable> TableReader<'_, Tb> {
    pub fn get_row(&self, key: &Tb::Key) -> Option<Tb::Value> {
        match self.tables.get_table_handle::<Tb>() {
            Ok(handle) => {
                if let Some(tb) = self.table.get(handle.id()) {
                    tb.get_row(key)
                } else {
                    None
                }
            }
            Err(e) => {
                warn!("table handle not found: {}", e);
                None
            }
        }
    }
}

impl<Tb: OneTable> TableReader<'_, Tb> {
    pub fn get_data(&self) -> Option<Tb::Value> {
        match self.tables.get_table_handle::<Tb>() {
            Ok(handle) => self.table.get(handle.id()).map(|tb| tb.get_data()),
            Err(e) => {
                warn!("table handle not found: {}", e);
                None
            }
        }
    }
}

impl<'w, Tb: NotIndexListTable> TableReader<'w, Tb> {
    pub fn iter(&self) -> Option<impl Iterator<Item = &Tb::Value>> {
        match self.tables.get_table_handle::<Tb>() {
            Ok(handle) => self.table.get(handle.id()).map(|tb| tb.iter()),
            Err(e) => {
                warn!("table handle not found: {}", e);
                None
            }
        }
    }
}

impl<'w, Tb: MultiUnionIndexListTable> TableReader<'w, Tb> {
    pub fn get_row_by_key(&self, key: &Tb::Key) -> Option<Tb::Value> {
        match self.tables.get_table_handle::<Tb>() {
            Ok(handle) => {
                if let Some(tb) = self.table.get(handle.id()) {
                    tb.get_row_by_key(key)
                } else {
                    None
                }
            }
            Err(e) => {
                warn!("table handle not found: {}", e);
                None
            }
        }
    }
}

impl<'w, Tb: MultiIndexListTable> TableReader<'w, Tb> {
    pub fn get_row_by(&self, key: &Tb::Key) -> Option<Tb::Value> {
        match self.tables.get_table_handle::<Tb>() {
            Ok(handle) => {
                if let Some(tb) = self.table.get(handle.id()) {
                    tb.get_row_by(key)
                } else {
                    None
                }
            }
            Err(e) => {
                warn!("table handle not found: {}", e);
                None
            }
        }
    }
}
