use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

use crate::ecology::EcologyType;

use super::EcologyMaterial;

#[derive(AssetCollection, Resource, Debug)]
pub struct ForestEcologyMaterial {
    #[asset(key = "grass.color")]
    #[asset(optional)]
    pub base_color_texture: Option<Handle<Image>>,
    #[asset(key = "grass.normal_map")]
    #[asset(optional)]
    pub normal_texture: Option<Handle<Image>>,
    #[asset(key = "grass.ambient_occlusion")]
    #[asset(optional)]
    pub occlusion_texture: Option<Handle<Image>>,
    #[asset(key = "grass.metallic_roughness")]
    #[asset(optional)]
    pub metallic_roughness_texture: Option<Handle<Image>>,
    #[asset(key = "grass.depth")]
    #[asset(optional)]
    pub depth_texture: Option<Handle<Image>>,
}

impl EcologyMaterial for ForestEcologyMaterial {
    fn get_ecology_type(&self) -> EcologyType {
        EcologyType::Forest
    }

    fn get_base_color_texture(&self) -> Option<Handle<Image>> {
        self.base_color_texture.clone()
    }

    fn get_normal_texture(&self) -> Option<Handle<Image>> {
        self.normal_texture.clone()
    }

    fn get_occlusion_texture(&self) -> Option<Handle<Image>> {
        self.occlusion_texture.clone()
    }

    fn get_metallic_roughness_texture(&self) -> Option<Handle<Image>> {
        self.metallic_roughness_texture.clone()
    }

    fn get_depth_texture(&self) -> Option<Handle<Image>> {
        self.depth_texture.clone()
    }
}
