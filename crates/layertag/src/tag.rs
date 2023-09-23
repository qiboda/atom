use std::{borrow::Cow, ops::Deref};

use bevy::prelude::Name;

#[derive(Debug, PartialEq, Eq)]
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
