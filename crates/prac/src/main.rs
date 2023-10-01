use std::ops::{Deref, DerefMut};

#[derive(Default, Debug, PartialEq, Eq)]
struct A {
    pub a: i32,
}

impl DerefMut for A {
    fn deref_mut(&mut self) -> &mut Self::Target {
        println!("DerefMut");
        &mut self.a
    }
}

impl Deref for A {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        println!("Deref");
        &self.a
    }
}

fn main() {
    let a = A::default();

    assert!(*a == 0);
}
