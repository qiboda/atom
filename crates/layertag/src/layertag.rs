use std::{
    fmt::Debug,
    ops::{ControlFlow, Deref, DerefMut},
};

use bevy::reflect::{reflect_trait, FromReflect, Reflect, TypePath};

use crate::tag::Tag;

#[reflect_trait]
pub trait LayerTagClone {
    fn box_clone(&self) -> Box<dyn LayerTag>;
}

pub trait LayerTagCounter {
    fn increase_count(&mut self);

    fn decrease_count(&mut self);

    fn count(&self) -> usize;

    fn reset_count(&mut self);
}

/// A tag with data.
#[reflect_trait]
pub trait LayerTagData: Reflect + Debug + LayerTagClone + LayerTagCounter {
    fn cmp_data_same_type_inner(&self, rhs: &dyn LayerTag) -> bool;
}

/// A tag that can be used to identify a layer, and with struct data.
#[reflect_trait]
pub trait LayerTag: LayerTagData {
    fn tag(&self) -> &[Tag];

    /// two tag exact match
    /// for example:
    /// ```ignore
    /// "a.b.c" == "a.b.c"
    /// "a.b" != "a.b.c"
    /// ```
    fn exact_match(&self, rhs: &dyn LayerTag) -> bool {
        self.tag() == rhs.tag()
    }

    /// two tag exact match
    /// for example
    /// ```ignore
    /// "a.b.c" == "a.b.c"
    /// "a.b" == "a.b.c"
    /// "a.b.d" != "a.b.c.d"
    /// ```
    fn partical_match(&self, rhs: &dyn LayerTag) -> bool {
        let r = self
            .tag()
            .iter()
            .zip(rhs.tag().iter())
            .try_for_each(|(x, y)| {
                if x == y {
                    return ControlFlow::Continue(());
                }
                ControlFlow::Break(())
            });

        r == ControlFlow::Continue(())
    }

    /// get same prefix
    /// for example
    /// ```ignore
    /// "a.b.c" -> "a.b.d" -> "a.b"
    /// "a.b.c" -> "a.b.c" -> "a.b.c"
    /// "" -> "a.b.c" -> ""
    /// ```
    fn same_prefix(&self, rhs: &dyn LayerTag) -> Vec<&Tag> {
        let mut ret = vec![];
        self.tag()
            .iter()
            .zip(rhs.tag().iter())
            .try_for_each(|(x, y)| {
                if x == y {
                    ret.push(x);
                    return ControlFlow::Continue(());
                }
                ControlFlow::Break(())
            });
        ret
    }

    /// campare tag data only same tag.
    fn cmp_data_same_type(&self, rhs: &dyn LayerTag) -> bool {
        assert!(self.tag() == rhs.tag());
        self.cmp_data_same_type_inner(rhs)
    }
}

impl TypePath for Box<dyn LayerTag> {
    fn type_path() -> &'static str {
        core::concat!(
            core::concat!(core::module_path!(), "::"),
            "Box<dyn LayerTag>"
        )
    }

    fn short_type_path() -> &'static str {
        "Box<dyn LayerTag>"
    }

    fn type_ident() -> Option<&'static str> {
        Some("Box<dyn LayerTag>")
    }

    fn crate_name() -> Option<&'static str> {
        Some(core::module_path!().split(':').next().unwrap())
    }

    fn module_path() -> Option<&'static str> {
        Some(core::module_path!())
    }
}

impl Reflect for Box<dyn LayerTag> {
    fn type_name(&self) -> &str {
        core::any::type_name::<Self>()
    }

    fn get_represented_type_info(&self) -> Option<&'static bevy::reflect::TypeInfo> {
        self.deref().get_represented_type_info()
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn into_reflect(self: Box<Self>) -> Box<dyn Reflect> {
        self
    }

    fn as_reflect(&self) -> &dyn Reflect {
        self
    }

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        self
    }

    fn apply(&mut self, value: &dyn Reflect) {
        self.deref_mut().apply(value)
    }

    fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
        *self = <dyn Reflect>::take(value)?;
        Ok(())
    }

    fn reflect_ref(&self) -> bevy::reflect::ReflectRef {
        self.deref().reflect_ref()
    }

    fn reflect_mut(&mut self) -> bevy::reflect::ReflectMut {
        self.deref_mut().reflect_mut()
    }

    fn reflect_owned(self: Box<Self>) -> bevy::reflect::ReflectOwned {
        // ReflectOwned::Value(self)
        self.deref().box_clone().reflect_owned()
    }

    fn reflect_hash(&self) -> Option<u64> {
        self.deref().reflect_hash()
    }

    fn reflect_partial_eq(&self, _value: &dyn Reflect) -> Option<bool> {
        self.deref().reflect_partial_eq(_value)
    }

    fn serializable(&self) -> Option<bevy::reflect::serde::Serializable> {
        None
    }

    fn is_dynamic(&self) -> bool {
        false
    }

    fn clone_value(&self) -> Box<dyn Reflect> {
        self.deref().clone_value()
    }
}

impl FromReflect for Box<dyn LayerTag> {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let v = reflect.downcast_ref::<Box<dyn LayerTag>>()?;
        Some(v.box_clone())
    }
}

#[cfg(test)]
mod test {
    use core::fmt;

    use bevy::reflect::Reflect;

    extern crate self as layertag;
    use layertag_derive::LayerTag;

    use crate::tag::Tag;

    use super::{LayerTag, LayerTagData};

    #[derive(LayerTag, Reflect, Debug, Clone)]
    #[layer_tag("a", "b", "c")]
    struct TestTag;

    impl LayerTagData for TestTag {
        fn cmp_data_same_type_inner(&self, _rhs: &dyn crate::layertag::LayerTag) -> bool {
            true
        }
    }

    impl fmt::Display for TestTag {
        fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt::Result::Ok(())
        }
    }

    #[derive(LayerTag, Reflect, Debug, Clone)]
    #[layer_tag("a", "b", "d")]
    struct TestTag2 {}

    impl LayerTagData for TestTag2 {
        fn cmp_data_same_type_inner(&self, _rhs: &dyn LayerTag) -> bool {
            true
        }
    }

    #[derive(LayerTag, Reflect, Debug, Clone)]
    #[layer_tag("a", "b")]
    struct TestTag3 {}

    impl LayerTagData for TestTag3 {
        fn cmp_data_same_type_inner(&self, _rhs: &dyn LayerTag) -> bool {
            true
        }
    }

    #[allow(clippy::bool_assert_comparison)]
    #[test]
    fn cmp_layer_tag() {
        let test_tag = TestTag {};
        let test_tag_dup = TestTag {};
        let test_tag_2 = TestTag2 {};
        let test_tag_3 = TestTag3 {};

        assert_eq!(test_tag_dup.exact_match(&test_tag), true);
        assert_eq!(test_tag.exact_match(&test_tag_2), false);

        assert_eq!(test_tag.partical_match(&test_tag_2), false);
        assert_eq!(test_tag.partical_match(&test_tag_3), true);

        assert_eq!(
            test_tag.same_prefix(&test_tag_2),
            vec![&Tag::new("a"), &Tag::new("b")]
        );

        assert_eq!(
            test_tag.same_prefix(&test_tag_3),
            vec![&Tag::new("a"), &Tag::new("b")]
        );
    }
}
