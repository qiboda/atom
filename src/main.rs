use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(keyboard_change_color)
        .add_system(animate_sprite)
        .run();
}

fn keyboard_change_color(keyboard_input: Res<Input<KeyCode>>, mut sprite: Query<&mut Sprite>) {
    if keyboard_input.just_pressed(KeyCode::A) {
        for mut s in sprite.iter_mut() {
            s.color = Color::RED;
        }
    }

    if keyboard_input.just_released(KeyCode::A) {
        for mut s in sprite.iter_mut() {
            s.color = Color::WHITE;
        }
    }
}

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        // &mut Transform,
        // &mut GlobalTransform,
    )>,
) {
    // for (indices, mut timer, mut sprite, mut transform, mut global_transform) in &mut query {
    //     println!(
    //         "index: {}, transform {:?}, global_transform: {:?}",
    //         sprite.index, transform, global_transform
    //     );
    // }
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = if sprite.index == indices.last {
                indices.first
            } else {
                sprite.index + 1
            };
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("output.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(600.0, 586.0), 1, 48, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    // Use only the subset of sprites in the sheet that make up the run animation
    let animation_indices = AnimationIndices { first: 0, last: 47 };

    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite::new(animation_indices.first),
            // transform: Transform::from_scale(Vec3::splat(6.0)),
            ..default()
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
    commands.spawn(SpriteBundle {
        texture: asset_server.load("screenshot_jiumeizi.png"),
        ..default()
    });
}
