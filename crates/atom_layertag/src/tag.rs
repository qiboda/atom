use std::{borrow::Cow, ops::Deref};

use bevy::{prelude::Name, reflect::Reflect};

/// 标签片段：包装 Bevy 的 [`Name`]，表示图层标签路径中的单个节点。
#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect, Hash)]
pub struct Tag(Name);

impl Tag {
    /// 从任意可转为静态字符串的值创建 [`Tag`]。
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

/// 批量创建 [`Tag`] 的便捷宏，展开为 `Vec<Tag>`。
///
/// 示例：
/// ```ignore
/// let tags = tags!["a", "b", "c"];
/// ```
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
