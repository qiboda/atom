use bevy_asset_loader::prelude::*;

use bevy::prelude::*;
use category::forest::{ForestEcologyMaterial, GrassEcologyMaterial};

use crate::TerrainState;

pub mod category;
pub mod ecology_set;
pub mod layer;

#[derive(Debug)]
pub enum EcologyType {
    Forest,
    Desert,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum TerrainMaterialLoadState {
    #[default]
    Wait,
    AssetLoading,
    AssetReinterpret,
    Next,
}

#[derive(Debug, Default)]
pub struct EcologyPlugin;

impl Plugin for EcologyPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<TerrainMaterialLoadState>()
            .add_loading_state(
                LoadingState::new(TerrainMaterialLoadState::AssetLoading)
                    .continue_to_state(TerrainMaterialLoadState::AssetReinterpret)
                    .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                        "textures/terrain/terrain.assets.ron",
                    )
                    .load_collection::<GrassEcologyMaterial>()
                    .load_collection::<ForestEcologyMaterial>(),
            )
            .add_systems(OnEnter(TerrainState::LoadAssets), to_load_assets)
            .add_systems(
                OnEnter(TerrainMaterialLoadState::AssetReinterpret),
                reinterpret_texture_array,
            )
            .add_systems(
                OnEnter(TerrainMaterialLoadState::Next),
                to_next_terrain_state,
            );
    }
}

fn to_load_assets(
    mut state: ResMut<NextState<TerrainMaterialLoadState>>,
    material_state: Res<State<TerrainMaterialLoadState>>,
) {
    if *material_state == TerrainMaterialLoadState::Wait {
        state.set(TerrainMaterialLoadState::AssetLoading);
    }
}

fn to_next_terrain_state(
    state: Res<State<TerrainState>>,
    mut next_state: ResMut<NextState<TerrainState>>,
) {
    if *state == TerrainState::LoadAssets {
        next_state.set(TerrainState::GenerateTerrainInfoMap);
    }
}

fn reinterpret_texture_array(
    mut images: ResMut<Assets<Image>>,
    material: Res<GrassEcologyMaterial>,
    mut next_state: ResMut<NextState<TerrainMaterialLoadState>>,
) {
    if let Some(image) = images.get_mut(
        material
            .base_color_texture
            .as_ref()
            .map_or(AssetId::invalid(), |x| x.id()),
    ) {
        image.reinterpret_stacked_2d_as_array(3);
    }

    if let Some(image) = images.get_mut(
        material
            .normal_texture
            .as_ref()
            .map_or(AssetId::invalid(), |x| x.id()),
    ) {
        image.reinterpret_stacked_2d_as_array(3);
    }

    if let Some(image) = images.get_mut(
        material
            .metallic_roughness_texture
            .as_ref()
            .map_or(AssetId::invalid(), |x| x.id()),
    ) {
        image.reinterpret_stacked_2d_as_array(3);
    }

    if let Some(image) = images.get_mut(
        material
            .depth_texture
            .as_ref()
            .map_or(AssetId::invalid(), |x| x.id()),
    ) {
        image.reinterpret_stacked_2d_as_array(3);
    }

    if let Some(image) = images.get_mut(
        material
            .occlusion_texture
            .as_ref()
            .map_or(AssetId::invalid(), |x| x.id()),
    ) {
        info!("ao image is valid");
        image.reinterpret_stacked_2d_as_array(3);
    }

    next_state.set(TerrainMaterialLoadState::Next);
}
