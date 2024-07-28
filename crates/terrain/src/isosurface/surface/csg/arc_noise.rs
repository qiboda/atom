use std::sync::Arc;

use noise::NoiseFn;

#[derive(Clone)]
pub struct ArcNoise<T, const DIM: usize> {
    source: Arc<dyn NoiseFn<T, DIM> + Sync + Send>,
}

impl<T, const DIM: usize> ArcNoise<T, DIM> {
    pub fn new(noise: impl NoiseFn<T, DIM> + Send + Sync + 'static) -> Self {
        Self {
            source: Arc::new(noise),
        }
    }
}

impl<T, const DIM: usize> NoiseFn<T, DIM> for ArcNoise<T, DIM> {
    fn get(&self, point: [T; DIM]) -> f64 {
        self.source.get(point)
    }
}
