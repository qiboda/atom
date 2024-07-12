use std::ops::Not;

use bevy::prelude::*;

use crate::setting::TerrainSetting;
use terrain_core::chunk::coords::TerrainChunkCoord;

use super::visible_range::VisibleTerrainRange;

#[derive(Debug, Default, Clone, Reflect)]
pub struct TerrainSingleVisibleArea {
    pub center_chunk_coord: TerrainChunkCoord,
    pub cached_min_chunk_coord: TerrainChunkCoord,
    pub cached_max_chunk_coord: TerrainChunkCoord,
}

impl TerrainSingleVisibleArea {
    pub fn iter_chunk(&self, callback: &mut impl FnMut(i64, i64, i64)) {
        for x in self.cached_min_chunk_coord.x..self.cached_max_chunk_coord.x {
            for y in self.cached_min_chunk_coord.y..self.cached_max_chunk_coord.y {
                for z in self.cached_min_chunk_coord.z..self.cached_max_chunk_coord.z {
                    callback(x, y, z);
                }
            }
        }
    }

    pub fn is_in_chunk(&self, x: i64, y: i64, z: i64) -> bool {
        (self.cached_min_chunk_coord.x..self.cached_max_chunk_coord.x).contains(&x)
            && (self.cached_min_chunk_coord.y..self.cached_max_chunk_coord.y).contains(&y)
            && (self.cached_min_chunk_coord.z..self.cached_max_chunk_coord.z).contains(&z)
    }
}

#[derive(Debug, Default, Clone, Component, Reflect)]
pub struct TerrainSingleVisibleAreaProxy {
    /// default is old
    pub terrain_single_visible_area_a: TerrainSingleVisibleArea,
    /// default is new
    pub terrain_single_visible_area_b: TerrainSingleVisibleArea,
    pub current_is_a: bool,
}

impl TerrainSingleVisibleAreaProxy {
    pub fn new(new_single_visible_area: &TerrainSingleVisibleArea) -> Self {
        Self {
            terrain_single_visible_area_a: TerrainSingleVisibleArea::default(),
            terrain_single_visible_area_b: new_single_visible_area.clone(),
            current_is_a: false,
        }
    }

    pub fn set_current(&mut self, new_single_visible_area: &TerrainSingleVisibleArea) {
        self.current_is_a = !self.current_is_a;
        if self.current_is_a {
            self.terrain_single_visible_area_a = new_single_visible_area.clone();
        } else {
            self.terrain_single_visible_area_b = new_single_visible_area.clone();
        }
    }

    pub fn remove_current(&mut self) {
        if self.current_is_a {
            self.terrain_single_visible_area_a = TerrainSingleVisibleArea::default();
        } else {
            self.terrain_single_visible_area_b = TerrainSingleVisibleArea::default();
        }
        self.current_is_a = !self.current_is_a;
    }

    pub fn get_current(&self) -> &TerrainSingleVisibleArea {
        if self.current_is_a {
            &self.terrain_single_visible_area_a
        } else {
            &self.terrain_single_visible_area_b
        }
    }

    pub fn get_last(&self) -> &TerrainSingleVisibleArea {
        if self.current_is_a {
            &self.terrain_single_visible_area_b
        } else {
            &self.terrain_single_visible_area_a
        }
    }
}

pub fn add_terrain_visible_areas(
    mut commands: Commands,
    visible_range: Query<
        Entity,
        (
            Added<VisibleTerrainRange>,
            Without<TerrainSingleVisibleAreaProxy>,
        ),
    >,
) {
    for entity in visible_range.iter() {
        commands
            .entity(entity)
            .insert(TerrainSingleVisibleAreaProxy::default());
    }
}

// #[bevycheck::system]
#[allow(clippy::type_complexity)]
pub fn update_terrain_visible_areas(
    terrain_settings: Res<TerrainSetting>,
    mut visible_range_query: Query<
        (
            &mut TerrainSingleVisibleAreaProxy,
            &Camera,
            &GlobalTransform,
            &VisibleTerrainRange,
        ),
        (
            Or<(
                Changed<GlobalTransform>,
                Changed<VisibleTerrainRange>,
                Changed<Camera>,
            )>,
        ),
    >,
) {
    for (mut visible_area, camera, global_transform, visible_range) in
        visible_range_query.iter_mut()
    {
        if camera.is_active.not() {
            let area = TerrainSingleVisibleArea::default();
            visible_area.set_current(&area);
            continue;
        }

        let camera_position = global_transform.translation();
        let chunk_size = terrain_settings.chunk_settings.chunk_size;

        let center_coord = camera_position / chunk_size;
        let min_coord = (camera_position + visible_range.min()) / chunk_size;
        let max_coord = (camera_position + visible_range.max()) / chunk_size;
        info!(
            "min_coord: {:?} max_coord: {:?}, visible_range: {:?}",
            min_coord, max_coord, visible_range
        );

        visible_area.set_current(&TerrainSingleVisibleArea {
            center_chunk_coord: TerrainChunkCoord::new(
                center_coord.x.floor() as i64,
                center_coord.y.floor() as i64,
                center_coord.z.floor() as i64,
            ),
            cached_min_chunk_coord: TerrainChunkCoord::new(
                min_coord.x.floor() as i64,
                min_coord.y.floor() as i64,
                min_coord.z.floor() as i64,
            ),
            cached_max_chunk_coord: TerrainChunkCoord::new(
                max_coord.x.floor() as i64,
                max_coord.y.floor() as i64,
                max_coord.z.floor() as i64,
            ),
        });
    }
}

/// Entity can not query when receive RemovedComponents if despawn entity.
pub fn remove_terrain_visible_areas(
    mut removed_range: RemovedComponents<VisibleTerrainRange>,
    mut removed_camera: RemovedComponents<Camera>,
    mut query: Query<(&mut TerrainSingleVisibleAreaProxy, Option<&Camera>)>,
) {
    removed_range.read().for_each(|removed_entity| {
        if let Ok((mut visible_area, Some(camera))) = query.get_mut(removed_entity) {
            if camera.is_active {
                let area = TerrainSingleVisibleArea::default();
                visible_area.set_current(&area);
            }
        }
    });

    removed_camera.read().for_each(|removed_entity| {
        if let Ok((mut visible_area, _camera)) = query.get_mut(removed_entity) {
            let area = TerrainSingleVisibleArea::default();
            visible_area.set_current(&area);
        }
    });
}
