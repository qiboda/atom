use std::{borrow::Cow, ops::Deref};

use bevy::{prelude::Name, reflect::Reflect};

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect, Hash)]
pub struct Tag(Name);

impl Tag {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self(Name::new(name))
    }
}

impl Deref for Tag {
    type Target = Name;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[macro_export]
macro_rules! tags {
    [$($name:expr),*] => {
        vec![$(Tag::new($name),)*]
    };
}

#[cfg(test)]
mod tests {
    use crate::tag::Tag;

    #[test]
    pub fn test_macro() {
        let a = tags!["a", "b", "aslkdfj"];
        assert!(a.len() == 3);
        assert_eq!(a, vec![Tag::new("a"), Tag::new("b"), Tag::new("aslkdfj"),]);
    }
}
