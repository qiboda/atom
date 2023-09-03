pub mod forest;

use std::fmt::Debug;

use bevy::prelude::{Handle, Image};

use super::EcologyType;

pub trait EcologyMaterial: Send + Sync + Debug {
    fn get_ecology_type(&self) -> EcologyType;

    fn get_albedo_texture(&self) -> Handle<Image>;
    fn get_normal_texture(&self) -> Handle<Image>;
    fn get_occlusion_texture(&self) -> Handle<Image>;
    fn get_metallic_texture(&self) -> Handle<Image>;
    fn get_roughness_texture(&self) -> Handle<Image>;
    fn get_height_texture(&self) -> Handle<Image>;
}
