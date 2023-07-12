pub mod seg_component;

use bevy::prelude::*;

use self::seg_component::{edit_transitional_face, generate_segments, trace_comonent};

use super::{octree::OctreePlugin, IsosurfaceExtractionSet};

pub struct ExtractPluign;

impl Plugin for ExtractPluign {
    fn build(&self, app: &mut App) {
        app.add_plugins(OctreePlugin).add_systems(
            Startup,
            (generate_segments, edit_transitional_face, trace_comonent)
                .in_set(IsosurfaceExtractionSet::Extract),
        );
    }
}
