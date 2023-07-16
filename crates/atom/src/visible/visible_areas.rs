use bevy::prelude::*;
use bevy::utils::HashMap;

use crate::terrain::chunk::coords::TerrainChunkCoord;
use crate::terrain::settings::TerrainSettings;

use super::visible::VisibleTerrainRange;

#[derive(Debug, Default, Clone)]
pub struct TerrainSingleVisibleArea {
    pub global_transform: GlobalTransform,
    pub cached_min_chunk_coord: TerrainChunkCoord,
    pub cached_max_chunk_coord: TerrainChunkCoord,
}

impl TerrainSingleVisibleArea {
    pub fn iter_chunk(&self, callback: &mut impl FnMut(i64, i64, i64) -> ()) {
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

#[derive(Debug, Default, Clone)]
pub struct TerrainSingleVisibleAreaProxy {
    /// default is old
    pub terrain_singel_visible_area_a: TerrainSingleVisibleArea,
    /// default is new
    pub terrain_singel_visible_area_b: TerrainSingleVisibleArea,
    pub current_is_a: bool,
}

impl TerrainSingleVisibleAreaProxy {
    pub fn new(new_single_visible_area: &TerrainSingleVisibleArea) -> Self {
        Self {
            terrain_singel_visible_area_a: TerrainSingleVisibleArea::default(),
            terrain_singel_visible_area_b: new_single_visible_area.clone(),
            current_is_a: false,
        }
    }

    pub fn set_current(&mut self, new_single_visible_area: &TerrainSingleVisibleArea) {
        self.current_is_a = !self.current_is_a;
        if self.current_is_a {
            self.terrain_singel_visible_area_a = new_single_visible_area.clone();
        } else {
            self.terrain_singel_visible_area_b = new_single_visible_area.clone();
        }
    }

    pub fn remove_current(&mut self) {
        if self.current_is_a {
            self.terrain_singel_visible_area_a = TerrainSingleVisibleArea::default();
        } else {
            self.terrain_singel_visible_area_b = TerrainSingleVisibleArea::default();
        }
        self.current_is_a = !self.current_is_a;
    }

    pub fn get_current(&self) -> &TerrainSingleVisibleArea {
        if self.current_is_a {
            &self.terrain_singel_visible_area_a
        } else {
            &self.terrain_singel_visible_area_b
        }
    }

    pub fn get_last(&self) -> &TerrainSingleVisibleArea {
        if self.current_is_a {
            &self.terrain_singel_visible_area_b
        } else {
            &self.terrain_singel_visible_area_a
        }
    }
}

#[derive(Debug, Resource, Default)]
pub struct TerrainVisibleAreas {
    visible_area_proxys: HashMap<Entity, TerrainSingleVisibleAreaProxy>,
}

impl TerrainVisibleAreas {
    pub fn set_current_visible_area(
        &mut self,
        entity: Entity,
        new_single_visible_area: &TerrainSingleVisibleArea,
    ) {
        match self.visible_area_proxys.get_mut(&entity) {
            Some(proxy) => proxy.set_current(new_single_visible_area),
            None => {
                self.visible_area_proxys.insert(
                    entity,
                    TerrainSingleVisibleAreaProxy::new(new_single_visible_area),
                );
            }
        }
    }

    pub fn remove_current_visible_area(&mut self, entity: Entity) {
        match self.visible_area_proxys.get_mut(&entity) {
            Some(proxy) => {
                proxy.remove_current();
            }
            None => {}
        }
    }

    pub fn get_last_visible_area(&self, entity: Entity) -> Option<&TerrainSingleVisibleArea> {
        match self.visible_area_proxys.get(&entity) {
            Some(proxy) => Some(proxy.get_last()),
            None => None,
        }
    }

    pub fn get_current_visible_area(&self, entity: Entity) -> Option<&TerrainSingleVisibleArea> {
        match self.visible_area_proxys.get(&entity) {
            Some(proxy) => Some(proxy.get_current()),
            None => None,
        }
    }

    pub fn get_all_current_visible_area(&self) -> Vec<&TerrainSingleVisibleArea> {
        self.visible_area_proxys
            .values()
            .map(|proxy| proxy.get_current())
            .collect()
    }

    pub fn get_all_last_visible_area(&self) -> Vec<&TerrainSingleVisibleArea> {
        self.visible_area_proxys
            .values()
            .map(|proxy| proxy.get_last())
            .collect()
    }
}

// #[bevycheck::system]
pub fn update_terrain_visible_areas(
    mut terrain_centers: ResMut<TerrainVisibleAreas>,
    terrain_settings: Res<TerrainSettings>,
    visible_range_query: Query<
        (Entity, &GlobalTransform, &VisibleTerrainRange),
        (
            Or<(Changed<GlobalTransform>, Changed<VisibleTerrainRange>)>,
            With<VisibleTerrainRange>,
        ),
    >,
) {
    for (entity, global_transform, visible_range) in visible_range_query.iter() {
        let camera_position = global_transform.translation();

        let camera_position = Vec3::new(0.0, 0.0, 0.0);

        let chunk_size = terrain_settings.get_chunk_size();

        let min_coord = (camera_position + visible_range.min) / chunk_size;
        let max_coord = (camera_position + visible_range.max) / chunk_size;
        info!(
            "min_coord: {:?} max_coord: {:?}, visible_range: {:?}",
            min_coord, max_coord, visible_range
        );

        terrain_centers.set_current_visible_area(
            entity,
            &TerrainSingleVisibleArea {
                global_transform: *global_transform,
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
            },
        );
    }
}

pub fn remove_terrain_visible_areas(
    mut terrain_centers: ResMut<TerrainVisibleAreas>,
    mut removed_events: RemovedComponents<VisibleTerrainRange>,
) {
    removed_events.iter().for_each(|removed_entity| {
        terrain_centers.remove_current_visible_area(removed_entity);
    });
}
