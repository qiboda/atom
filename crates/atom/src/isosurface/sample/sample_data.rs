use bevy::prelude::UVec3;

#[derive(Default, Clone, Debug)]
pub struct SampleData<T> {
    data: Vec<T>,

    // coord size
    size: UVec3,
}

impl<T> SampleData<T>
where
    T: Default + Clone,
{
    pub fn new(size: UVec3) -> Self {
        SampleData {
            data: vec![T::default(); (size.x * size.y * size.z) as usize],
            size,
        }
    }

    #[inline]
    pub fn resize(&mut self, size: UVec3) {
        self.data
            .resize((size.x * size.y * size.z) as usize, T::default());
        self.size = size;
    }
}

impl<T> SampleData<T> {
    // from xyz to data index
    #[inline]
    pub fn get_data_index(&self, coord: UVec3) -> usize {
        ((coord.x * self.size.y + coord.y) * self.size.z + coord.z) as usize
    }
}

impl<T> SampleData<T>
where
    T: Clone,
{
    #[inline]
    pub fn set_value(&mut self, coord: UVec3, value: T) {
        let index = self.get_data_index(coord);
        self.data[index] = value;
    }

    pub fn set_all_values(&mut self, values: Vec<T>) {
        self.data = values;
    }

    #[inline]
    pub fn get_value(&self, coord: UVec3) -> T {
        self.data[self.get_data_index(coord) as usize].clone()
    }

    #[inline]
    pub fn get_size(&self) -> UVec3 {
        self.size
    }
}
