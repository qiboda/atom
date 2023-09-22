use std::any::TypeId;

use bevy::{prelude::Component, utils::HashMap};

use crate::layertag::LayerTag;

pub trait FromTagRegistry {
    fn from_tag_registry() -> Self;
}

/// register layer tag.
/// 1. support query layer tag from tag type struct or tag name.
#[derive(Default, Component)]
pub struct LayerTagRegistry {
    layers: HashMap<TypeId, Box<dyn LayerTag>>,
}

impl LayerTagRegistry {
    pub fn register<T>(&mut self)
    where
        T: LayerTag + FromTagRegistry,
    {
        self.layers
            .insert(TypeId::of::<T>(), Box::new(T::from_tag_registry()));
    }

    pub fn get<T: LayerTag>(&self) -> Option<&T> {
        self.layers
            .get(&TypeId::of::<T>())
            .map(|x| x.as_reflect().downcast_ref::<T>().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use bevy::reflect::Reflect;

    use crate::layertag::LayerTag;

    use super::{FromTagRegistry, LayerTagRegistry};

    #[derive(Reflect)]
    struct TestTag {}

    impl FromTagRegistry for TestTag {
        fn from_tag_registry() -> Self {
            Self {}
        }
    }

    impl LayerTag for TestTag {
        fn tag(&self) -> &[crate::tag::Tag] {
            &[]
        }
    }

    #[derive(Reflect)]
    struct TestTag2 {}

    impl FromTagRegistry for TestTag2 {
        fn from_tag_registry() -> Self {
            Self {}
        }
    }

    impl LayerTag for TestTag2 {
        fn tag(&self) -> &[crate::tag::Tag] {
            &[]
        }
    }

    #[test]
    fn register_layertag() {
        let mut registry = LayerTagRegistry::default();
        registry.register::<TestTag>();
        assert!(registry.get::<TestTag>().is_some());
        assert!(registry.get::<TestTag2>().is_none());
    }
}
