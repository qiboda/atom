use bevy::{prelude::*, reflect::TypePath};
use layertag::layertag::LayerTag;
use std::{fmt::Debug, marker::PhantomData};

use layertag_derive::{layer_tag, LayerTag};

#[derive(LayerTag, Clone, Debug, Reflect)]
#[layer_tag("a", "b", "c")]
pub struct TestTags {}

static TAG_A: &str = "a";
static TAG_B: &str = "b";

#[derive(LayerTag, Debug, Clone, Reflect)]
#[layer_tag(TAG_A, TAG_B, "d")]
pub struct GenTestTags<T>
where
    T: Default + Reflect + TypePath + Debug,
{
    #[reflect(ignore)]
    _data: PhantomData<T>,
}

fn main() {
    println!("--- test_tags ---");
    let test_tags = TestTags {};
    for tag in test_tags.tag().iter() {
        println!("tag: {:?}", tag);
    }
    println!("--- generic_tags ---");
    let generic_tags = GenTestTags::<i32> { _data: PhantomData };
    for tag in generic_tags.tag().iter() {
        println!("tag: {:?}", tag);
    }
}
