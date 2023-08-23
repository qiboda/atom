use bevy::math::Vec3A;

#[derive(Clone, Debug, Default)]
pub struct CellExtent {
    min: Vec3A,
    max: Vec3A,
}

impl CellExtent {
    pub fn new(min: Vec3A, max: Vec3A) -> Self {
        Self { min, max }
    }
}

impl CellExtent {
    pub fn corners(&self) -> [Vec3A; 8] {
        [
            self.min,
            Vec3A::new(self.max.x, self.min.y, self.min.z),
            Vec3A::new(self.min.x, self.max.y, self.min.z),
            Vec3A::new(self.max.x, self.max.y, self.min.z),
            Vec3A::new(self.min.x, self.min.y, self.max.z),
            Vec3A::new(self.max.x, self.min.y, self.max.z),
            Vec3A::new(self.min.x, self.max.y, self.max.z),
            self.max,
        ]
    }

    pub fn center(&self) -> Vec3A {
        (self.min + self.max) / 2.0
    }

    pub fn min(&self) -> &Vec3A {
        &self.min
    }

    pub fn max(&self) -> &Vec3A {
        &self.max
    }

    pub fn size(&self) -> Vec3A {
        self.max - self.min
    }

    pub fn split(&self, split: Vec3A) -> [CellExtent; 8] {
        [
            Self::new(self.min, split),
            Self::new(
                Vec3A::from([split.x, self.min.y, self.min.z]),
                Vec3A::from([self.max.x, split.y, split.z]),
            ),
            Self::new(
                Vec3A::from([self.min.x, split.y, self.min.z]),
                Vec3A::from([split.x, self.max.y, split.z]),
            ),
            Self::new(
                Vec3A::from([split.x, split.y, self.min.z]),
                Vec3A::from([self.max.x, self.max.y, split.z]),
            ),
            Self::new(
                Vec3A::from([self.min.x, self.min.y, split.z]),
                Vec3A::from([split.x, split.y, self.max.z]),
            ),
            Self::new(
                Vec3A::from([split.x, self.min.y, split.z]),
                Vec3A::from([self.max.x, split.y, self.max.z]),
            ),
            Self::new(
                Vec3A::from([self.min.x, split.y, split.z]),
                Vec3A::from([split.x, self.max.y, self.max.z]),
            ),
            Self::new(split, self.max),
        ]
    }
}
