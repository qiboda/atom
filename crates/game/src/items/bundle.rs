use bevy::prelude::*;

use super::{
    comp::{ItemCd, ItemCount, ItemStack},
    ItemRow,
};

#[derive(Debug, Bundle, Default)]
pub struct ItemBase {
    pub row: ItemRow,
    pub count: ItemCount,
    pub cd: ItemCd,
    pub stack: ItemStack,
}
