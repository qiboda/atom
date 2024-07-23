use bevy::{
    app::Update,
    log::{debug, info},
};
use std::sync::Arc;

use bevy::prelude::{Added, App, AssetServer, Commands, Entity, Plugin, Query, Res, Startup};

use crate::chunk_mgr::chunk::bundle::TerrainChunk;

use self::{
    category::forest::ForestEcologyMaterial,
    ecology_set::EcologyMaterials,
    layer::{first::FirstLayer, EcologyLayerSampler},
};

pub mod category;
pub mod ecology_set;
pub mod layer;

#[derive(Debug)]
pub enum EcologyType {
    Forest,
    Desert,
}

#[derive(Debug, Default)]
pub struct EcologyPlugin;

impl Plugin for EcologyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup)
            .add_systems(Update, add_ecology_layer_sampler);
    }
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("startup ecology");

    let albedo_image_handle = asset_server
        .load("texture/terrain/M3D_RockyAridGround01_4K/RockyAridGround01_ALBEDO_4K.png");
    let ao_image_handle =
        asset_server.load("texture/terrain/M3D_RockyAridGround01_4K/RockyAridGround01_AO_4K.png");
    let height_image_handle = asset_server
        .load("texture/terrain/M3D_RockyAridGround01_4K/RockyAridGround01_HEIGHT_4K.png");
    let metallic_image_handle = asset_server
        .load("texture/terrain/M3D_RockyAridGround01_4K/RockyAridGround01_METALLIC_4K.png");
    let normal_image_handle = asset_server
        .load("texture/terrain/M3D_RockyAridGround01_4K/RockyAridGround01_NORMAL_4K.png");
    let rough_image_handle = asset_server
        .load("texture/terrain/M3D_RockyAridGround01_4K/RockyAridGround01_ROUGH_4K.png");

    commands.insert_resource(EcologyMaterials {
        forest_material: Arc::new(ForestEcologyMaterial {
            albedo_texture: albedo_image_handle.clone(),
            normal_texture: normal_image_handle.clone(),
            clussion_texture: ao_image_handle.clone(),
            metallic_texture: metallic_image_handle.clone(),
            roughness_texture: rough_image_handle.clone(),
            height_texture: height_image_handle.clone(),
        }),
        desert_material: Arc::new(ForestEcologyMaterial {
            albedo_texture: albedo_image_handle,
            normal_texture: normal_image_handle,
            clussion_texture: ao_image_handle,
            metallic_texture: metallic_image_handle,
            roughness_texture: rough_image_handle,
            height_texture: height_image_handle,
        }),
    });
}

fn add_ecology_layer_sampler(
    mut commands: Commands,
    terrain_query: Query<Entity, Added<TerrainChunk>>,
    ecology_materials: Res<EcologyMaterials>,
) {
    for entity in terrain_query.iter() {
        debug!(
            "add_ecology_layer_sampler: entity count: {}",
            terrain_query.iter().count()
        );
        if let Some(mut cmds) = commands.get_entity(entity) {
            cmds.try_insert(EcologyLayerSampler {
                all_layer: vec![
                    Box::new(FirstLayer {
                        forest_material: ecology_materials.forest_material.clone(),
                    }),
                    Box::new(FirstLayer {
                        forest_material: ecology_materials.desert_material.clone(),
                    }),
                ],
            });
        }
    }
}
