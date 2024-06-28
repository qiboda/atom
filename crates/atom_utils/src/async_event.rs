use std::marker::PhantomData;
use std::ops::Not;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Mutex;

use bevy::ecs::event::event_update_system;
use bevy::prelude::*;

#[derive(Resource, Deref, DerefMut)]
struct EventChannelReceiver<T>(Mutex<Receiver<T>>);

#[derive(Resource, Deref, DerefMut)]
pub struct EventChannelSender<T>(Mutex<Sender<T>>);

#[derive(Default, Debug)]
pub struct AsyncEventPlugin<T>(PhantomData<T>);

impl<T: Event> Plugin for AsyncEventPlugin<T> {
    fn build(&self, app: &mut App) {
        assert!(
            app.world()
                .contains_resource::<EventChannelReceiver<T>>()
                .not(),
            "this event channel is already initialized",
        );

        let (sender, recv) = std::sync::mpsc::channel::<T>();

        app.add_event::<T>()
            .add_systems(First, (channel_to_event::<T>,).after(event_update_system))
            .insert_resource(EventChannelSender(Mutex::new(sender)))
            .insert_resource(EventChannelReceiver(Mutex::new(recv)));
    }
}

fn channel_to_event<T: Event>(receiver: Res<EventChannelReceiver<T>>, mut writer: EventWriter<T>) {
    // this should be the only system working with the receiver,
    // thus we always expect to get this lock
    let events = receiver.lock().expect("unable to acquire mutex lock");

    writer.send_batch(events.try_iter());
}
