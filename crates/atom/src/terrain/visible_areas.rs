use bevy::prelude::*;
use bevy::utils::HashMap;

use super::data::coords::{TerrainGlobalCoord, VoxelGradedCoord};
use super::data::visible::VisibleTerrainRange;
use super::TerrainSystemSet;

#[derive(Default, Debug)]
pub struct TerrainVisibleAreaPlugin;

impl Plugin for TerrainVisibleAreaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_terrain_visible_areas, remove_terrain_visible_areas)
                .chain()
                .in_set(TerrainSystemSet::VisibleAreas),
        );
    }
}

#[derive(Debug, Default, Clone)]
pub struct TerrainSingleVisibleArea {
    pub global_transform: GlobalTransform,
    pub cached_min_voxle_graded_coord: VoxelGradedCoord,
    pub cached_max_voxle_graded_coord: VoxelGradedCoord,
}

impl TerrainSingleVisibleArea {
    pub fn iter_chunk(&self, callback: &mut impl FnMut(i64, i64, i64) -> ()) {
        for x in self.cached_min_voxle_graded_coord.chunk_coord.x
            ..self.cached_max_voxle_graded_coord.chunk_coord.x
        {
            // for y in self.cached_min_voxle_graded_coord.chunk_coord.y
            //     ..self.cached_max_voxle_graded_coord.chunk_coord.y
            // {
            for z in self.cached_min_voxle_graded_coord.chunk_coord.z
                ..self.cached_max_voxle_graded_coord.chunk_coord.z
            {
                callback(x, 0, z);
            }
            // }
        }
    }

    pub fn is_in_chunk(&self, x: i64, _y: i64, z: i64) -> bool {
        (self.cached_min_voxle_graded_coord.chunk_coord.x
            ..self.cached_max_voxle_graded_coord.chunk_coord.x)
            .contains(&x)
            // && (self.cached_min_voxle_graded_coord.chunk_coord.y
            //     ..self.cached_max_voxle_graded_coord.chunk_coord.y)
            //     .contains(&y)
            && (self.cached_min_voxle_graded_coord.chunk_coord.z
                ..self.cached_max_voxle_graded_coord.chunk_coord.z)
                .contains(&z)
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

        let min_global_coord =
            TerrainGlobalCoord::from_location(&(camera_position + visible_range.min));
        // todo: add one offset
        let max_global_coord =
            TerrainGlobalCoord::from_location(&(camera_position + visible_range.max));

        let min_graded_coord = VoxelGradedCoord::from(&min_global_coord);
        let max_graded_coord = VoxelGradedCoord::from(&max_global_coord);

        terrain_centers.set_current_visible_area(
            entity,
            &TerrainSingleVisibleArea {
                global_transform: *global_transform,
                cached_min_voxle_graded_coord: min_graded_coord,
                cached_max_voxle_graded_coord: max_graded_coord,
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
