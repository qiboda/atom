pub mod persist;
pub mod plugin;
pub mod setting_path;
pub mod toml_diff;
pub mod load;

use bevy::prelude::*;

use serde::{Deserialize, Serialize};

/// settings limits:
///   1. all fields must be Optional
pub trait Settings:
    Resource + Copy + Serialize + TypePath + Default + for<'a> Deserialize<'a> + Asset
{
}

impl<T> Settings for T where
    T: Resource + Copy + Serialize + TypePath + Default + for<'a> Deserialize<'a> + Asset
{
}
#[cfg(test)]
mod tests {
    use bevy::{asset::Asset, reflect::TypePath};
    use serde::{Deserialize, Serialize};
    use serde_merge::tmerge;

    #[derive(Serialize, Deserialize, PartialEq, Debug, Asset, TypePath)]
    struct TestSettings {
        a: Option<u32>,
        b: Option<String>,
    }

    #[test]
    fn merge() {
        let a = TestSettings {
            a: Some(1),
            b: Some("a".to_string()),
        };

        let b = TestSettings {
            a: Some(1),
            b: Some("b".to_string()),
        };

        let merge = tmerge(&a, &b).unwrap();
        assert_eq!(b, merge);
    }
}
