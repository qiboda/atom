use bevy::{core::FrameCount, diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};

pub struct FrameUIPlugin;
impl Plugin for FrameUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            // .add_plugins(LogDiagnosticsPlugin::default())
            .add_systems(Startup, add_frame_ui)
            .add_systems(Update, update_fps);
    }
}

#[derive(Default, Component)]
struct FpsText;

fn add_frame_ui(mut commands: Commands) {
    commands
        .spawn(TextBundle {
            text: Text {
                sections: vec![
                    TextSection {
                        value: "fps: ".to_string(),
                        style: TextStyle {
                            font_size: 60.0,
                            color: Color::BLUE,
                            ..default()
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font_size: 60.0,
                            color: Color::RED,
                            ..default()
                        },
                    },
                    TextSection {
                        value: "frame count: ".to_string(),
                        style: TextStyle {
                            font_size: 60.0,
                            color: Color::BLUE,
                            ..default()
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font_size: 60.0,
                            color: Color::GREEN,
                            ..default()
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font_size: 60.0,
                            color: Color::GREEN,
                            ..default()
                        },
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(FpsText);
}

fn update_fps(
    time: Res<Time>,
    frame_count: Res<FrameCount>,
    mut query: Query<&mut Text, With<FpsText>>,
    mut camera: Query<&mut GlobalTransform, With<Camera>>,
) {
    let mut text = query.single_mut();
    text.sections[1].value = format!("{:.2}", 1.0 / time.delta_seconds());
    text.sections[3].value = format!("{:.2}", frame_count.0);
    text.sections[4].value = format!("{:?}", camera.single_mut().translation());
}
