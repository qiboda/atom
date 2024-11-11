use std::fmt::Debug;

pub trait SwapDataItemTrait {
    type Item;

    fn insert(&mut self, data: Self::Item);

    fn remove(&mut self, data: &Self::Item);

    fn clear(&mut self);

    fn batch_insert(&mut self, data: Vec<Self::Item>);
}

pub trait SwapDataTakeTrait {
    type Item: SwapDataItemTrait;

    fn take_last(&mut self) -> Self::Item;
    fn take_current(&mut self) -> Self::Item;
}

pub trait SwapDataTrait {
    type Item: SwapDataItemTrait;

    fn get_current(&self) -> &Self::Item;

    fn get_current_mut(&mut self) -> &mut Self::Item;

    fn get_last(&self) -> &Self::Item;

    fn get_last_mut(&mut self) -> &mut Self::Item;

    fn swap(&mut self);

    fn insert(&mut self, data: <Self::Item as SwapDataItemTrait>::Item) {
        self.get_current_mut().insert(data);
    }

    fn remove(&mut self, data: &<Self::Item as SwapDataItemTrait>::Item) {
        self.get_current_mut().remove(data);
    }

    fn extend(&mut self, data: Vec<<Self::Item as SwapDataItemTrait>::Item>) {
        self.get_current_mut().batch_insert(data);
    }
}

// Example implementation
pub struct SwapData<T> {
    a: T,
    b: T,
    current_a: bool,
}

impl<T> Default for SwapData<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            a: T::default(),
            b: T::default(),
            current_a: true,
        }
    }
}

impl<T> SwapDataTrait for SwapData<T>
where
    T: SwapDataItemTrait,
{
    type Item = T;

    fn get_current(&self) -> &T {
        if self.current_a {
            &self.a
        } else {
            &self.b
        }
    }

    fn get_current_mut(&mut self) -> &mut Self::Item {
        if self.current_a {
            &mut self.a
        } else {
            &mut self.b
        }
    }

    fn swap(&mut self) {
        self.current_a = !self.current_a;
        if self.current_a {
            self.a.clear();
        } else {
            self.b.clear();
        }
    }

    fn get_last(&self) -> &Self::Item {
        if self.current_a {
            &self.b
        } else {
            &self.a
        }
    }

    fn get_last_mut(&mut self) -> &mut Self::Item {
        if self.current_a {
            &mut self.b
        } else {
            &mut self.a
        }
    }
}

impl<T> SwapDataTakeTrait for SwapData<T>
where
    T: SwapDataItemTrait + Default,
{
    type Item = T;

    fn take_last(&mut self) -> Self::Item {
        if self.current_a {
            std::mem::take(&mut self.b)
        } else {
            std::mem::take(&mut self.a)
        }
    }

    fn take_current(&mut self) -> Self::Item {
        if self.current_a {
            std::mem::take(&mut self.a)
        } else {
            std::mem::take(&mut self.b)
        }
    }
}

impl<T> SwapDataItemTrait for Vec<T>
where
    T: PartialEq + Debug,
{
    type Item = T;

    fn insert(&mut self, data: Self::Item) {
        self.push(data);
    }

    fn remove(&mut self, data: &Self::Item) {
        if let Some(index) = self.iter().position(|x| x == data) {
            self.remove(index);
        }
    }

    fn batch_insert(&mut self, data: Vec<Self::Item>) {
        self.extend(data);
    }

    fn clear(&mut self) {
        self.clear();
    }
}
