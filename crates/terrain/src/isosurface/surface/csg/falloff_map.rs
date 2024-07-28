use num_traits::float::TotalOrder;

use noise::NoiseFn;

use num_traits::Float;

#[derive(Clone, Copy)]
pub struct MapEdge<T, const DIM: usize>
where
    T: Float,
{
    half_size: [T; DIM],
    inner_half_size: [T; DIM],
    a: T,
    b: T,
}

impl<T, const DIM: usize> MapEdge<T, DIM>
where
    T: Float,
    f64: Into<T>,
{
    pub fn new() -> Self {
        Self {
            half_size: [100.0.into(); DIM],
            inner_half_size: [80.0.into(); DIM],
            a: 3.0.into(),
            b: 2.0.into(),
        }
    }

    pub fn with_half_size(mut self, size: [T; DIM]) -> Self {
        self.half_size = size;
        self
    }

    pub fn with_inner_half_size(mut self, size: [T; DIM]) -> Self {
        self.inner_half_size = size;
        self
    }

    pub fn with_a_b(mut self, a: T, b: T) -> Self {
        self.a = a;
        self.b = b;
        self
    }
}

impl<T, const DIM: usize> MapEdge<T, DIM>
where
    T: Float,
{
    // input and output range is [0, 1]
    pub(crate) fn smoothstep(&self, value: T) -> T {
        value * value * (self.a - self.b * value)
    }
}

impl<T, const DIM: usize> NoiseFn<T, DIM> for MapEdge<T, DIM>
where
    T: Float + TotalOrder + Into<f64>,
    f64: Into<T>,
{
    fn get(&self, point: [T; DIM]) -> f64 {
        // range is [0, 1]
        let offset = point
            .iter()
            .zip(self.half_size.iter().zip(self.inner_half_size.iter()))
            .map(|(&p, (&s, &i))| {
                let p = p.abs();
                if p < i {
                    0.0.into()
                } else {
                    (p - i) / (s - i)
                }
            });

        // range is [0, 1], center is 0 and edge is 1
        let value = offset.max_by(|a, b| a.total_cmp(b)).unwrap();

        -self.smoothstep(value).into() * 2.0
    }
}
