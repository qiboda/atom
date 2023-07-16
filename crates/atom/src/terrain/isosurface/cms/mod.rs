pub mod seg_component;

use bevy::prelude::*;

use self::seg_component::{edit_transitional_face, generate_segments, trace_comonent};

use super::IsosurfaceExtractionSet;

pub struct ExtractPlugin;

impl Plugin for ExtractPlugin {
    fn build(&self, app: &mut App) {
        info!("add ExtractPluign");
        app.add_systems(
            Update,
            (generate_segments, edit_transitional_face, trace_comonent)
                .chain()
                .in_set(IsosurfaceExtractionSet::Extract),
        );
    }
}
