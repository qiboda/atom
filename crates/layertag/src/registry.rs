use std::any::TypeId;

use bevy::{prelude::Component, reflect::Reflect, utils::HashMap};

use crate::layertag::LayerTag;

pub trait FromTagRegistry {
    fn from_tag_registry() -> Self;
}

/// register layer tag.
/// 1. support query layer tag from tag type struct or tag name.
#[derive(Default, Component, Debug)]
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

impl LayerTagRegistry {
    pub fn request<T: LayerTag + Clone>(&self) -> Option<T> {
        let layertag = self.get::<T>();
        layertag.cloned()
    }
}

#[cfg(test)]
mod tests {
    use std::fmt;

    use bevy::reflect::Reflect;
    extern crate self as layertag;
    use layertag_derive::LayerTag;

    use crate::layertag::{LayerTag, LayerTagData};

    use super::{FromTagRegistry, LayerTagRegistry};

    #[derive(LayerTag, Reflect, Debug, Clone, PartialEq, Eq)]
    #[layer_tag()]
    struct TestTag {
        pub value: i32,
    }

    impl FromTagRegistry for TestTag {
        fn from_tag_registry() -> Self {
            Self { value: 0 }
        }
    }

    impl LayerTagData for TestTag {
        fn cmp_data_same_type_inner(&self, rhs: &dyn LayerTag) -> bool {
            assert_eq!(self.tag(), rhs.tag());

            if let Some(rhs) = rhs.as_reflect().downcast_ref::<Self>() {
                self.value == rhs.value
            } else {
                false
            }
        }
    }

    impl fmt::Display for TestTag {
        fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
            Ok(())
        }
    }

    #[derive(Reflect, Debug, Clone, LayerTag)]
    struct TestTag2 {}

    impl FromTagRegistry for TestTag2 {
        fn from_tag_registry() -> Self {
            Self {}
        }
    }

    impl fmt::Display for TestTag2 {
        fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
            Ok(())
        }
    }

    #[test]
    fn register_layertag() {
        let mut registry = LayerTagRegistry::default();
        registry.register::<TestTag>();
        assert!(registry.get::<TestTag>().is_some());
        assert!(registry.get::<TestTag2>().is_none());
    }

    #[test]
    fn request_layertag() {
        let mut registry = LayerTagRegistry::default();
        registry.register::<TestTag>();
        let new_tag_inst = registry.request::<TestTag>();
        assert!(new_tag_inst.is_some());
        assert_eq!(new_tag_inst, Some(TestTag::from_tag_registry()));
        assert!(registry.request::<TestTag2>().is_none());

        let mut new_tag_inst_2 = registry.request::<TestTag>();
        new_tag_inst_2.as_mut().map(|v| {
            v.value = 3;
            v
        });
        assert_ne!(new_tag_inst_2, registry.request::<TestTag>());
    }
}
