use std::fmt::Debug;

pub trait IsoSurface {
    fn get_value(&self, x: f32, y: f32, z: f32) -> f32;

    /// iso level is the value that where the iso surface will be generated at
    /// if the iso level is 0.5, the iso surface will be generated at the middle of the
    /// volume(voxel)
    /// 0 is left, 1 is right and 0.5 is half.
    fn set_iso_level(&mut self, iso_level: f32);

    fn get_iso_level(&self) -> f32;

    /// if negative_inside is true, the iso surface will be generated inside the volume
    fn set_negative_inside(&mut self, negative_inside: bool);

    fn is_negative_inside(&self) -> bool;
}

impl Debug for dyn IsoSurface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IsoSurface").finish()
    }
}
