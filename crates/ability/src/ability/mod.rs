pub trait Ability {
    fn cast();

    fn commit();

    fn finish();

    fn abort();

    fn pause();

    fn resume();
}

pub enum AbilityCommand {
    Start,
    Clear,
    Abort,
    Pause,
    Resume,
}
