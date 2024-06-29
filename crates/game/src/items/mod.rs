pub mod bundle;
pub mod category;
pub mod comp;
pub mod inventory;
pub mod plugin;

/// 背包，仓库，宝箱。物品栏等。
///
///
/// 显示所有的物品。
/// 判断类型。
/// CD
/// 数量
/// 最大数量
///
/// 是否可堆叠
/// 是否可使用
/// 是否可装备
/// 是否可拾取
/// 是否可丢弃
/// 是否可交易
/// 是否可出售
/// 是否可购买
/// 是否可合成
/// 是否可分解
/// 是否可修理
/// 是否可升级
/// 是否可附魔
///
///
///
use std::{fmt::Debug, sync::Arc};

use bevy::prelude::*;
use cfg::{item::TbItemKey, Item};

#[derive(Debug, Default, Component)]
pub struct ItemRow {
    pub key: TbItemKey,
    pub data: Option<Arc<Item>>,
}
