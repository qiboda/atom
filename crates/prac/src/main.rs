use bevy::prelude::*;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (despawn_entity, /*apply_deferred,*/ add_child).chain(),
        )
        .run();
}

#[derive(Component)]
struct ComponentA;

#[derive(Component)]
struct ComponentB;

fn startup(mut commands: Commands) {
    commands.spawn(ComponentA);
}

fn despawn_entity(mut commands: Commands, query: Query<Entity, With<ComponentA>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn add_child(mut commands: Commands, query: Query<Entity, With<ComponentA>>) {
    for entity in query.iter() {
        let child = commands.spawn(ComponentB).id();
        commands.entity(entity).add_child(child);
    }
}
