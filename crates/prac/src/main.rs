use bevy::core::FrameCount;
use bevy::prelude::*;
use bevy::DefaultPlugins;

#[derive(Event, Debug)]
struct MyEvent {}

fn main() {
    let mut app = bevy::app::App::new();
    app.add_plugins(DefaultPlugins)
        .add_event::<MyEvent>()
        .add_systems(Update, (update_event, update_event_two))
        .run();
}

fn update_event(mut events: ResMut<Events<MyEvent>>, frame_count: Res<FrameCount>) {
    for event in events.get_reader().iter(&events) {
        info!("events read: {:?}, {}", event, frame_count.0);
    }

    events.send(MyEvent {});
}

fn update_event_two(mut events: ResMut<Events<MyEvent>>, frame_count: Res<FrameCount>) {
    for event in events.get_reader().iter(&events) {
        info!("events two read: {:?}, {}", event, frame_count.0);
    }

    events.send(MyEvent {});
}
