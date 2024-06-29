use bevy::prelude::*;

// 是角色或者其他实体的child
#[derive(Debug, Default, Component)]
pub struct ItemSet {
    pub items: Vec<Entity>,
}

// item component in inventory
#[derive(Debug, Component, PartialEq, Eq)]
pub struct InInventory {
    pub inventory: Entity,
}

impl InInventory {
    pub fn new(inventory: Entity) -> Self {
        Self { inventory }
    }
}

#[derive(Debug, Default, Component)]
pub struct Inventory;

#[derive(Debug, Default, Bundle)]
pub struct InventoryBundle {
    pub item_set: ItemSet,
    pub inventory: Inventory,
}
