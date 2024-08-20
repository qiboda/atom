use bevy::prelude::{Handle, Image};

use crate::ecology::EcologyType;

use super::EcologyMaterial;

#[derive(Debug)]
pub struct ForestEcologyMaterial {
    pub albedo_texture: Handle<Image>,
    pub normal_texture: Handle<Image>,
    pub occlusion_texture: Handle<Image>,
    pub metallic_texture: Handle<Image>,
    pub roughness_texture: Handle<Image>,
    pub height_texture: Handle<Image>,
}

impl EcologyMaterial for ForestEcologyMaterial {
    fn get_ecology_type(&self) -> EcologyType {
        EcologyType::Forest
    }

    fn get_albedo_texture(&self) -> Handle<Image> {
        self.albedo_texture.clone()
    }

    fn get_normal_texture(&self) -> Handle<Image> {
        self.normal_texture.clone()
    }

    fn get_occlusion_texture(&self) -> Handle<Image> {
        self.occlusion_texture.clone()
    }

    fn get_metallic_texture(&self) -> Handle<Image> {
        self.metallic_texture.clone()
    }

    fn get_roughness_texture(&self) -> Handle<Image> {
        self.roughness_texture.clone()
    }

    fn get_height_texture(&self) -> Handle<Image> {
        self.height_texture.clone()
    }
}
