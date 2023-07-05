pub mod seg_component;
pub mod tessellation;

use bevy::prelude::*;

use self::seg_component::{edit_transitional_face, generate_segments, trace_comonent};

use super::IsosurfaceExtractionSet;

#[derive(Debug, Clone, Eq, PartialEq, Hash, SystemSet)]
pub enum CMSSet {
    Octree,
    Algorithm,
}

pub struct ExtractPluign;

impl Plugin for ExtractPluign {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Startup,
            (CMSSet::Octree, CMSSet::Algorithm)
                .chain()
                .in_set(IsosurfaceExtractionSet::Extract),
        )
        .add_systems(
            Startup,
            (generate_segments, edit_transitional_face, trace_comonent).in_set(CMSSet::Algorithm),
        );
    }
}
