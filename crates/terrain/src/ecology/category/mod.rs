pub mod forest;

use std::fmt::Debug;

use bevy::prelude::{Handle, Image};

use super::EcologyType;

pub trait EcologyMaterial: Send + Sync + Debug {
    fn get_ecology_type(&self) -> EcologyType;

    fn get_base_color_texture(&self) -> Option<Handle<Image>>;
    fn get_normal_texture(&self) -> Option<Handle<Image>>;
    fn get_occlusion_texture(&self) -> Option<Handle<Image>>;
    fn get_metallic_roughness_texture(&self) -> Option<Handle<Image>>;
    fn get_depth_texture(&self) -> Option<Handle<Image>>;
}
