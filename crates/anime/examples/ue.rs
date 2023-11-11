use std::env;

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    // set env variable for asset path when debugging
    let key = "CARGO_MANIFEST_DIR";
    env::set_var(key, "E:/terrain/");

    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .init_resource::<AnimationClipSet>()
        .add_systems(Startup, start_up)
        .add_systems(Update, (play_animation, names))
        .run();
}

#[derive(Resource, Debug, Default)]
pub struct AnimationClipSet(pub Vec<Handle<AnimationClip>>);

fn start_up(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    _animation_clip_set: ResMut<AnimationClipSet>,
) {
    // info!(
    //     "path: {}, bevy: {}",
    //     env::var(key).unwrap(),
    //     env::var("BEVY_ASSET_PATH").unwrap()
    // );

    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0))
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..Default::default()
    });

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::new(4.0, 5.0, 4.0)),
        ..Default::default()
    });

    // let scene = asset_server.load("animations/ThirdPersonIdle.gltf#Scene0");
    // let material = asset_server.load("models/SK_Mannequin.gltf#Material0");

    let scene: Handle<Scene> = asset_server.load("animations/ThirdPersonIdle.gltf#Scene0");

    // commands.spawn((
    //     MaterialMeshBundle::<StandardMaterial> {
    //         mesh: scene,
    //         material,
    //         ..default()
    //     },
    //     AnimationPlayer::default(),
    // ));

    commands.spawn(SceneBundle {
        scene,
        ..Default::default()
    });
}

fn play_animation(
    input: Res<Input<KeyCode>>,
    animation_clip_set: Res<AnimationClipSet>,
    mut query: Query<&mut AnimationPlayer>,
) {
    if input.just_pressed(KeyCode::S) {
        info!("player num: {}", query.iter_mut().len());
        for mut player in query.iter_mut() {
            player
                .play(animation_clip_set.0.get(0).unwrap().clone())
                .repeat();
        }
    }
}

fn names(names: Query<&Name>) {
    for name in names.iter() {
        info!("name: {:?}", name);
    }
}
