use bevy::prelude::*;
use bevy::window::PresentMode;

pub fn toggle_vsync(
    input: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window>,
    mut commands: Commands,
    camera: Query<(Entity, &Camera)>,
) {
    if input.just_pressed(KeyCode::KeyV) {
        let mut window = windows.single_mut();

        window.present_mode = if matches!(window.present_mode, PresentMode::AutoVsync) {
            PresentMode::AutoNoVsync
        } else {
            PresentMode::AutoVsync
        };
        info!("PRESENT_MODE: {:?}", window.present_mode);
        camera.iter().all(|(entity, _)| {
            commands.entity(entity).despawn();
            true
        });
    }
}

fn exit_game(keyboard_input: Res<ButtonInput<KeyCode>>, mut app_exit_events: EventWriter<AppExit>) {
    if keyboard_input.just_released(KeyCode::Escape) {
        app_exit_events.send(AppExit::Success);
    }
}