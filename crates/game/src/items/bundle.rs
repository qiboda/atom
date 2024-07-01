use bevy::prelude::*;
use cfg::item::TbItemRow;

use super::comp::{ItemCd, ItemCount, ItemStack};

#[derive(Debug, Bundle, Default)]
pub struct ItemBase {
    pub row: TbItemRow,
    pub count: ItemCount,
    pub cd: ItemCd,
    pub stack: ItemStack,
}
