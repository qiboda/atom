pub mod active_camera_controller;
pub mod bundle;

use bevy::prelude::*;
use bundle::{ActiveCamera, ActiveCameraBundle};

/// add to a entity, to make active camera to follow it
#[derive(Component, Debug, Default)]
pub struct ActiveCameraFollowed;

#[derive(Debug, Default)]
pub struct CameraManagerPlugin;

impl Plugin for CameraManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraTracker>()
            .add_systems(Startup, spawn_active_camera)
            .add_systems(Update, active_camera_follow)
            .observe(on_add_camera)
            .observe(on_remove_camera);
    }
}

#[derive(Default, Debug, Resource)]
pub struct CameraTracker {
    pub cameras: Vec<Entity>,
    pub active_camera: Option<Entity>,
}

impl CameraTracker {
    pub fn get_active_camera(&self) -> Option<Entity> {
        self.active_camera
    }

    pub fn set_active_camera(&mut self, entity: Entity) {
        self.active_camera = Some(entity);
    }

    pub fn track_camera(&mut self, entity: Entity) {
        self.cameras.push(entity);
    }

    pub fn untrack_camera(&mut self, entity: Entity) {
        self.cameras.retain(|x| *x != entity);
        if self.active_camera == Some(entity) {
            self.active_camera = None;
        }
    }
}

fn spawn_active_camera(mut commands: Commands) {
    // Spawn view model camera.
    commands.spawn(ActiveCameraBundle::default());
}

fn active_camera_follow(
    followed: Query<&Transform, With<ActiveCameraFollowed>>,
    mut camera: Query<&mut Transform, With<ActiveCamera>>,
) {
    let followed_transform = followed.get_single();
    let camera_transform = camera.get_single_mut();
    match (followed_transform, camera_transform) {
        (Ok(followed_transform), Ok(mut camera_transform)) => {
            camera_transform.translation = followed_transform.translation;
        }
        (Ok(_), Err(_)) => {}
        (Err(_), Ok(_)) => {}
        (Err(_), Err(_)) => {}
    }
}

fn on_add_camera(
    trigger: Trigger<OnAdd, Camera>,
    query: Query<Has<ActiveCamera>, With<Camera>>,
    mut camera_tracker: ResMut<CameraTracker>,
) {
    camera_tracker.track_camera(trigger.entity());
    if let Ok(has_active_camera) = query.get(trigger.entity()) {
        if has_active_camera {
            camera_tracker.set_active_camera(trigger.entity());
        }
    }
}

fn on_remove_camera(trigger: Trigger<OnRemove, Camera>, mut camera_tracker: ResMut<CameraTracker>) {
    camera_tracker.untrack_camera(trigger.entity());
}
