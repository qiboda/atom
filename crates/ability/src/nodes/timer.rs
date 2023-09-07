use bevy::{
    prelude::{Component, Query, Res},
    time::Time,
};

#[derive(Clone, Debug, Default, Component)]
pub struct EffectNodeTimer {
    elapse: f32,
    duration: f32,
}

impl EffectNode for EffectNodeTimer {
    fn start(&mut self) {
        self.elapse = 0.0;
    }

    fn clear(&mut self) {
        self.elapse = 0.0;
        self.duration = f32::MAX;
    }
}

impl EffectDynamicNode for EffectNodeTimer {
    fn abort(&mut self) {
        self.clear();
    }

    fn pause(&mut self) {
        todo!()
    }

    fn resume(&mut self) {
        todo!()
    }

    fn update(&mut self) { }
}

fn update_timer(query: Query<(&mut EffectNodeTimer, &mut EffectNodeState)>, time: Res<Time>) {
    for (mut timer, mut state) in query.iter_mut() {
        match *state {
            EffectNodeState::Running => {
                timer.elapse += time.delta_seconds();
                if timer.elapse >= timer.duration {
                    *state = EffectNodeState::Finished;
                }
            }
            _ => {}
        }
    }
}
