use bevy::{prelude::*, reflect::TypePath};
use layertag::{
    layertag::{LayerTag, LayerTagData},
    registry::{FromTagRegistry, LayerTagRegistry},
};
use std::{fmt::Debug, marker::PhantomData};

use layertag_derive::LayerTag;

#[derive(LayerTag, Clone, Debug, Reflect)]
#[layer_tag("a", "b", "c")]
pub struct TestTags {
    #[layer_tag_counter]
    counter: usize,
}

static TAG_A: &str = "a";
static TAG_B: &str = "b";

impl FromTagRegistry for TestTags {
    fn from_tag_registry() -> Self {
        Self { counter: 0 }
    }
}

#[derive(LayerTag, Debug, Clone, Reflect)]
#[layer_tag(TAG_A, TAG_B, "d")]
pub struct GenTestTags<T>
where
    T: Default + Reflect + TypePath + Debug + Clone,
{
    #[reflect(ignore)]
    _data: PhantomData<T>,
}

impl<T> LayerTagData for GenTestTags<T>
where
    T: Default + Reflect + TypePath + Debug + Clone,
{
    #[doc = " campare tag data only same tag."]
    fn cmp_data_same_type_inner(&self, rhs: &dyn LayerTag) -> bool {
        assert!(self.tag() == rhs.tag());

        if let Some(rhs) = rhs.as_reflect().downcast_ref::<Self>() {
            self._data == rhs._data
        } else {
            false
        }
    }
}

fn main() {
    println!("--- test_tags ---");
    let mut layertag_registry = LayerTagRegistry::default();
    layertag_registry.register::<TestTags>();
    if let Some(test_tags) = layertag_registry.get::<TestTags>() {
        for tag in test_tags.tag().iter() {
            println!("tag: {:?}", tag);
        }
    }
    println!("--- generic_tags ---");
    let generic_tags = GenTestTags::<i32> { _data: PhantomData };
    for tag in generic_tags.tag().iter() {
        println!("tag: {:?}", tag);
    }
}
