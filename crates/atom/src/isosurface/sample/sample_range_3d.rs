use nalgebra::Vector3;

#[derive(Default, Clone, Debug)]
pub struct SampleRange3D<T> {
    data: Vec<T>,

    // coord size
    size: Vector3<usize>,

    // position(location) size
    pos_size: Vector3<(f32, f32)>,
}

impl<T> SampleRange3D<T>
where
    T: Default + Clone,
{
    pub fn new(pos_size: Vector3<(f32, f32)>, size: Vector3<usize>) -> Self {
        SampleRange3D {
            data: vec![T::default(); size.x * size.y * size.z],
            size,
            pos_size,
        }
    }

    #[inline]
    pub fn resize(&mut self, pos_size: Vector3<(f32, f32)>, size: Vector3<usize>) {
        self.data.resize(size.x * size.y * size.z, T::default());
        self.size = size;
        self.pos_size = pos_size;
    }
}

impl<T> SampleRange3D<T> {
    // from xyz to data index
    #[inline]
    pub fn get_data_index(&self, x: usize, y: usize, z: usize) -> usize {
        (x * self.size.y + y) * self.size.z + z
    }

    // from coord to position(location)
    #[inline]
    pub fn get_pos(&self, x: usize, y: usize, z: usize) -> Vector3<f32> {
        let tx = x as f32 / (self.size.x - 1) as f32;
        let ty = y as f32 / (self.size.y - 1) as f32;
        let tz = z as f32 / (self.size.z - 1) as f32;

        let mut pos = Vector3::new(tx, ty, tz);

        pos.x = self.pos_size[0].0 + (self.pos_size[0].1 - self.pos_size[0].0) * pos.x;
        pos.y = self.pos_size[0].0 + (self.pos_size[0].1 - self.pos_size[0].0) * pos.y;
        pos.z = self.pos_size[0].0 + (self.pos_size[0].1 - self.pos_size[0].0) * pos.z;

        pos
    }

    pub fn get_pos_size(&self) -> &Vector3<(f32, f32)> {
        &self.pos_size
    }
}

impl<T> SampleRange3D<T>
where
    T: Clone,
{
    #[inline]
    pub fn set_value(&mut self, x: usize, y: usize, z: usize, value: T) {
        let index = self.get_data_index(x, y, z);
        self.data[index] = value;
    }

    #[inline]
    pub fn get_value(&self, x: usize, y: usize, z: usize) -> T {
        self.data[self.get_data_index(x, y, z) as usize].clone()
    }
}
