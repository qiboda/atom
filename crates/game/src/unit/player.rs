use bevy::prelude::*;
use datatables::unit::TbPlayerRow;
use leafwing_input_manager::prelude::ActionState;
use lightyear::prelude::{client::Predicted, ClientId};
use serde::{Deserialize, Serialize};

use crate::input::setting::PlayerAction;

use super::base::{ClientUnitBundle, ServerUnitBundle};

#[derive(Debug, Component, Serialize, Deserialize, Clone, PartialEq)]
pub struct Player;

pub type PredictedPlayerFilter = (With<Predicted>, With<Player>);

#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct PlayerId(pub ClientId);

#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct BornLocation(pub Vec3);

#[derive(Bundle)]
pub struct ClientPlayerBundle {
    pub unit_bundle: ClientUnitBundle,
    pub action_state: ActionState<PlayerAction>,
}

impl Default for ClientPlayerBundle {
    fn default() -> Self {
        Self {
            unit_bundle: Default::default(),
            action_state: ActionState::<PlayerAction>::default(),
        }
    }
}

impl ClientPlayerBundle {
    pub fn new(radius: f32, length: f32) -> Self {
        Self {
            unit_bundle: ClientUnitBundle::new(radius, length),
            ..Default::default()
        }
    }
}

/// Server: 开头的，是服务器使用的，不能包括不replicate的组件。或者简单起见，可以只包含标记组件，用于客户端插入其他组件。
/// Client: 开头的，是客户端使用的，但可以包含需要replicate到客户端的组件，因为使用的host-server模式。
/// Server 和 Cilent 的 Bundle 中的组件不应该重叠。
#[derive(Bundle)]
pub struct ServerPlayerBundle {
    pub unit_bundle: ServerUnitBundle,
    pub born_location: BornLocation,
    pub tb_row: TbPlayerRow,
    pub player_id: PlayerId,
    pub player: Player,
}
