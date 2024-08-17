use bevy::{
    app::{Plugin, Update},
    prelude::{App, IntoSystemConfigs},
    render::extract_resource::ExtractResourcePlugin,
};

use crate::TerrainSystemSet;

use super::event::{read_csg_operation_apply_event, CSGOperateApplyEvent, CSGOperationRecords};

pub struct TerrainCSGPlugin;

impl Plugin for TerrainCSGPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CSGOperationRecords>()
            .add_plugins(ExtractResourcePlugin::<CSGOperationRecords>::default())
            .add_event::<CSGOperateApplyEvent>()
            .add_systems(
                Update,
                read_csg_operation_apply_event.in_set(TerrainSystemSet::ApplyCSG),
            );
    }
}
