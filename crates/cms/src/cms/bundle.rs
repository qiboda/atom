use bevy::prelude::Bundle;

use crate::sample::sample_info::SampleInfo;

use super::cms::CMSMeshInfo;

#[derive(Bundle, Default)]
pub struct CMSBundle {
    sample_info: SampleInfo,
    cms_mesh_info: CMSMeshInfo,
}
