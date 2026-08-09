//! Blackboard：图内节点间共享数据的黑板。

use bevy::{platform::collections::HashMap, prelude::*};
use std::borrow::Cow;

use bevy::{prelude::Entity, reflect::Reflect};

/// 图级共享数据容器组件：以 [`Name`] 为键存储 [`EffectValue`]。
#[derive(Debug, Component, Default)]
pub struct EffectBlackboard {
    /// 键 → 值映射。
    pub blackboard: HashMap<Name, EffectValue>,
}

#[allow(unused)]
/// 可存入 Blackboard 的运行时值：支持多种标量、实体引用与字符串。
///
/// `Clone`/`PartialEq` 为手写实现：`BoxReflect(Box<dyn Reflect>)` 中的 `Box<dyn Reflect>`
/// 无 std `Clone`/`PartialEq`（Bevy 0.19 不支持原生反射，issue #3392 未关闭），
/// 故按 `reflect_clone` / `reflect_partial_eq` 语义实现（见计划 §4.4）。
#[derive(Debug, Reflect)]
pub enum EffectValue {
    /// 8 位有符号整数。
    I8(i8),
    /// 16 位有符号整数。
    I16(i16),
    /// 32 位有符号整数。
    I32(i32),
    /// 64 位有符号整数。
    I64(i64),

    /// 8 位无符号整数。
    U8(u8),
    /// 16 位无符号整数。
    U16(u16),
    /// 32 位无符号整数。
    U32(u32),
    /// 64 位无符号整数。
    U64(u64),

    /// 32 位浮点数。
    F32(f32),
    /// 64 位浮点数。
    F64(f64),

    /// 单个实体引用。
    Entity(Entity),
    /// 实体引用列表。
    VecEntity(Vec<Entity>),

    /// 静态字符串（借用或拥有）。
    String(Cow<'static, str>),
    // Vec(Vec<EffectValue>),
    /// 反射值容器：任意实现 `Reflect` 的类型（如效果组件包）。
    ///
    /// `#[reflect(ignore)]` 排除反射 API；`default` 仅为满足 `FromReflect` 派生对
    /// `Box<dyn Reflect>` 的构造需求（正常路径不会触发该默认值）。
    BoxReflect(#[reflect(ignore, default = "box_reflect_default")] Box<dyn Reflect>),
}

/// `BoxReflect` 字段的 `FromReflect` 默认构造哨兵（实际使用中被手动构造覆盖）。
fn box_reflect_default() -> Box<dyn Reflect> {
    Box::new(())
}

impl Clone for EffectValue {
    fn clone(&self) -> Self {
        match self {
            EffectValue::I8(v) => EffectValue::I8(*v),
            EffectValue::I16(v) => EffectValue::I16(*v),
            EffectValue::I32(v) => EffectValue::I32(*v),
            EffectValue::I64(v) => EffectValue::I64(*v),
            EffectValue::U8(v) => EffectValue::U8(*v),
            EffectValue::U16(v) => EffectValue::U16(*v),
            EffectValue::U32(v) => EffectValue::U32(*v),
            EffectValue::U64(v) => EffectValue::U64(*v),
            EffectValue::F32(v) => EffectValue::F32(*v),
            EffectValue::F64(v) => EffectValue::F64(*v),
            EffectValue::Entity(v) => EffectValue::Entity(*v),
            EffectValue::VecEntity(v) => EffectValue::VecEntity(v.clone()),
            EffectValue::String(v) => EffectValue::String(v.clone()),
            EffectValue::BoxReflect(v) => EffectValue::BoxReflect(
                v.as_ref()
                    .reflect_clone()
                    .expect("BoxReflect 内部值必须可 reflect_clone"),
            ),
        }
    }
}

impl PartialEq for EffectValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (EffectValue::I8(a), EffectValue::I8(b)) => a == b,
            (EffectValue::I16(a), EffectValue::I16(b)) => a == b,
            (EffectValue::I32(a), EffectValue::I32(b)) => a == b,
            (EffectValue::I64(a), EffectValue::I64(b)) => a == b,
            (EffectValue::U8(a), EffectValue::U8(b)) => a == b,
            (EffectValue::U16(a), EffectValue::U16(b)) => a == b,
            (EffectValue::U32(a), EffectValue::U32(b)) => a == b,
            (EffectValue::U64(a), EffectValue::U64(b)) => a == b,
            (EffectValue::F32(a), EffectValue::F32(b)) => a == b,
            (EffectValue::F64(a), EffectValue::F64(b)) => a == b,
            (EffectValue::Entity(a), EffectValue::Entity(b)) => a == b,
            (EffectValue::VecEntity(a), EffectValue::VecEntity(b)) => a == b,
            (EffectValue::String(a), EffectValue::String(b)) => a == b,
            (EffectValue::BoxReflect(a), EffectValue::BoxReflect(b)) => {
                a.as_ref().reflect_partial_eq(b.as_ref()).unwrap_or(false)
            }
            _ => false,
        }
    }
}

/// 从 Blackboard 值中按类型取出引用（不可变/可变）。
///
/// 通过 `T: TryFrom<&Self>` / `T: TryFrom<&mut Self>` 实现类型安全的取值。
pub trait BlackBoardValue {
    /// 取出类型为 `T` 的不可变引用。
    fn get<'a, T>(&'a self) -> Result<T, T::Error>
    where
        T: TryFrom<&'a Self>;

    /// 取出类型为 `T` 的可变引用。
    fn get_mut<'a, T>(&'a mut self) -> Result<T, T::Error>
    where
        T: TryFrom<&'a mut Self>;
}

impl BlackBoardValue for EffectValue {
    fn get<'a, T>(&'a self) -> Result<T, T::Error>
    where
        T: TryFrom<&'a Self>,
    {
        self.try_into()
    }

    fn get_mut<'a, T>(&'a mut self) -> Result<T, T::Error>
    where
        T: TryFrom<&'a mut Self>,
    {
        self.try_into()
    }
}

impl BlackBoardValue for &EffectValue {
    fn get<'a, T>(&'a self) -> Result<T, T::Error>
    where
        T: TryFrom<&'a Self>,
    {
        self.try_into()
    }

    fn get_mut<'a, T>(&'a mut self) -> Result<T, T::Error>
    where
        T: TryFrom<&'a mut Self>,
    {
        self.try_into()
    }
}

impl<'a> TryFrom<&'a EffectValue> for &'a i8 {
    type Error = &'static str;

    fn try_from(value: &'a EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::I8(v) => Ok(v),
            _ => Err("not i8"),
        }
    }
}

impl<'a> TryFrom<&'a mut EffectValue> for &'a mut i8 {
    type Error = &'static str;

    fn try_from(value: &'a mut EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::I8(v) => Ok(v),
            _ => Err("not i8"),
        }
    }
}

impl<'a> TryFrom<&'a EffectValue> for &'a i16 {
    type Error = &'static str;

    fn try_from(value: &'a EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::I16(v) => Ok(v),
            _ => Err("not i16"),
        }
    }
}

impl<'a> TryFrom<&'a mut EffectValue> for &'a mut i16 {
    type Error = &'static str;

    fn try_from(value: &'a mut EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::I16(v) => Ok(v),
            _ => Err("not i16"),
        }
    }
}

impl<'a> TryFrom<&'a EffectValue> for &'a i32 {
    type Error = &'static str;

    fn try_from(value: &'a EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::I32(v) => Ok(v),
            _ => Err("not i32"),
        }
    }
}

impl<'a> TryFrom<&'a mut EffectValue> for &'a mut i32 {
    type Error = &'static str;

    fn try_from(value: &'a mut EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::I32(v) => Ok(v),
            _ => Err("not i32"),
        }
    }
}

impl<'a> TryFrom<&'a EffectValue> for &'a i64 {
    type Error = &'static str;

    fn try_from(value: &'a EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::I64(v) => Ok(v),
            _ => Err("not i64"),
        }
    }
}

impl<'a> TryFrom<&'a mut EffectValue> for &'a mut i64 {
    type Error = &'static str;

    fn try_from(value: &'a mut EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::I64(v) => Ok(v),
            _ => Err("not i64"),
        }
    }
}

impl<'a> TryFrom<&'a EffectValue> for &'a u8 {
    type Error = &'static str;

    fn try_from(value: &'a EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::U8(v) => Ok(v),
            _ => Err("not u8"),
        }
    }
}

impl<'a> TryFrom<&'a mut EffectValue> for &'a mut u8 {
    type Error = &'static str;

    fn try_from(value: &'a mut EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::U8(v) => Ok(v),
            _ => Err("not u8"),
        }
    }
}

impl<'a> TryFrom<&'a EffectValue> for &'a u16 {
    type Error = &'static str;

    fn try_from(value: &'a EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::U16(v) => Ok(v),
            _ => Err("not u16"),
        }
    }
}

impl<'a> TryFrom<&'a mut EffectValue> for &'a mut u16 {
    type Error = &'static str;

    fn try_from(value: &'a mut EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::U16(v) => Ok(v),
            _ => Err("not u16"),
        }
    }
}

impl<'a> TryFrom<&'a EffectValue> for &'a u32 {
    type Error = &'static str;

    fn try_from(value: &'a EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::U32(v) => Ok(v),
            _ => Err("not u32"),
        }
    }
}

impl<'a> TryFrom<&'a mut EffectValue> for &'a mut u32 {
    type Error = &'static str;

    fn try_from(value: &'a mut EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::U32(v) => Ok(v),
            _ => Err("not u32"),
        }
    }
}

impl<'a> TryFrom<&'a EffectValue> for &'a u64 {
    type Error = &'static str;

    fn try_from(value: &'a EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::U64(v) => Ok(v),
            _ => Err("not u64"),
        }
    }
}

impl<'a> TryFrom<&'a mut EffectValue> for &'a mut u64 {
    type Error = &'static str;

    fn try_from(value: &'a mut EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::U64(v) => Ok(v),
            _ => Err("not u64"),
        }
    }
}

impl<'a> TryFrom<&'a EffectValue> for &'a f32 {
    type Error = &'static str;

    fn try_from(value: &'a EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::F32(v) => Ok(v),
            _ => Err("not f32"),
        }
    }
}

impl<'a> TryFrom<&'a mut EffectValue> for &'a mut f32 {
    type Error = &'static str;

    fn try_from(value: &'a mut EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::F32(v) => Ok(v),
            _ => Err("not f32"),
        }
    }
}

impl<'a> TryFrom<&'a EffectValue> for &'a f64 {
    type Error = &'static str;

    fn try_from(value: &'a EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::F64(v) => Ok(v),
            _ => Err("not f64"),
        }
    }
}

impl<'a> TryFrom<&'a mut EffectValue> for &'a mut f64 {
    type Error = &'static str;

    fn try_from(value: &'a mut EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::F64(v) => Ok(v),
            _ => Err("not f64"),
        }
    }
}

impl<'a> TryFrom<&'a EffectValue> for String {
    type Error = &'static str;

    fn try_from(value: &'a EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::String(v) => Ok(v.to_string()),
            _ => Err("not String"),
        }
    }
}

impl<'a> TryFrom<&'a EffectValue> for &'a Cow<'static, str> {
    type Error = &'static str;

    fn try_from(value: &'a EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::String(v) => Ok(v),
            _ => Err("not String"),
        }
    }
}

impl<'a> TryFrom<&'a mut EffectValue> for &'a mut Cow<'static, str> {
    type Error = &'static str;

    fn try_from(value: &'a mut EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::String(v) => Ok(v),
            _ => Err("not String"),
        }
    }
}

impl<'a> TryFrom<&'a EffectValue> for &'a Vec<Entity> {
    type Error = &'static str;

    fn try_from(value: &'a EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::VecEntity(v) => Ok(v),
            _ => Err("not Vec"),
        }
    }
}

impl<'a> TryFrom<&'a mut EffectValue> for &'a mut Vec<Entity> {
    type Error = &'static str;

    fn try_from(value: &'a mut EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::VecEntity(v) => Ok(v),
            _ => Err("not Vec"),
        }
    }
}

// impl<'a, T> TryFrom<&'a EffectValue> for &'a Vec<T> {
//     type Error = &'static str;
//     fn try_from(value: &'a EffectValue) -> Result<Self, Self::Error> {
//         match value {
//             EffectValue::Vec(v) => {
//                 match v {
//                    v.into() as T => Ok(v)
//                 }
//             },
//             _ => Err("not Vec"),
//         }
//     }
// }

impl<'a> TryFrom<&'a EffectValue> for &'a Box<dyn Reflect> {
    type Error = &'static str;

    fn try_from(value: &'a EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::BoxReflect(v) => Ok(v),
            _ => Err("not BoxReflect"),
        }
    }
}

impl<'a> TryFrom<&'a mut EffectValue> for &'a mut Box<dyn Reflect> {
    type Error = &'static str;

    fn try_from(value: &'a mut EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::BoxReflect(v) => Ok(v),
            _ => Err("not BoxReflect"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    #[test]
    fn test_effect_value_i32() {
        let val = EffectValue::I32(42);
        assert_eq!(val.get::<&i32>(), Ok(&42));
    }

    #[test]
    fn test_effect_value_f32() {
        let val = EffectValue::F32(1.23);
        let result: Result<&f32, _> = (&val).try_into();
        assert_eq!(result, Ok(&1.23));
    }

    #[test]
    fn test_effect_value_string() {
        let val = EffectValue::String("hello".into());
        let result: Result<&Cow<'static, str>, _> = (&val).try_into();
        assert_eq!(result, Ok(&Cow::<'static, str>::Owned("hello".into())));
    }

    #[test]
    fn test_effect_value_wrong_type() {
        let val = EffectValue::I32(42);
        let result: Result<&f32, _> = (&val).try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_effect_value_get_mut() {
        let mut val = EffectValue::I32(100);
        *val.get_mut::<&mut i32>().expect("should be i32") = 200;
        assert_eq!(val.get::<&i32>(), Ok(&200));
    }

    #[test]
    fn test_effect_value_all_integer_types() {
        assert_eq!(EffectValue::I8(1).get::<&i8>(), Ok(&1i8));
        assert_eq!(EffectValue::I16(2).get::<&i16>(), Ok(&2i16));
        assert_eq!(EffectValue::I64(3).get::<&i64>(), Ok(&3i64));
        assert_eq!(EffectValue::U8(4).get::<&u8>(), Ok(&4u8));
        assert_eq!(EffectValue::U16(5).get::<&u16>(), Ok(&5u16));
        assert_eq!(EffectValue::U32(6).get::<&u32>(), Ok(&6u32));
        assert_eq!(EffectValue::U64(7).get::<&u64>(), Ok(&7u64));
    }

    #[test]
    fn test_effect_value_f64() {
        let val = EffectValue::F64(9.876);
        assert_eq!(val.get::<&f64>(), Ok(&9.876));
    }

    // ===== BoxReflect（BSN 迁移新增，见计划 §4.4）=====
    // Box<dyn Reflect> 无 std Clone/PartialEq，迁移后需手写实现（reflect_clone /
    // reflect_partial_eq 语义）——以下测试固化这些行为。

    #[derive(Debug, Reflect, PartialEq, Clone)]
    struct BoxReflectTestData {
        value: i32,
    }

    #[test]
    fn test_effect_value_box_reflect_clone() {
        let val = EffectValue::BoxReflect(Box::new(BoxReflectTestData { value: 42 }));
        let cloned = val.clone();

        let (EffectValue::BoxReflect(a), EffectValue::BoxReflect(b)) = (&val, &cloned) else {
            panic!("clone 后仍应为 BoxReflect 变体");
        };
        let a = a
            .as_ref()
            .downcast_ref::<BoxReflectTestData>()
            .expect("inner 类型必须可下转型");
        let b = b
            .as_ref()
            .downcast_ref::<BoxReflectTestData>()
            .expect("inner 类型必须可下转型");
        assert_eq!(
            a.value, b.value,
            "clone 必须按 reflect_clone 语义拷贝内部值"
        );
    }

    #[test]
    fn test_effect_value_box_reflect_partial_eq() {
        let a = EffectValue::BoxReflect(Box::new(BoxReflectTestData { value: 42 }));
        let b = EffectValue::BoxReflect(Box::new(BoxReflectTestData { value: 42 }));
        let c = EffectValue::BoxReflect(Box::new(BoxReflectTestData { value: 43 }));
        assert_eq!(a, b, "内部 reflect 值相等时 BoxReflect 应相等");
        assert_ne!(a, c, "内部 reflect 值不同时 BoxReflect 不应相等");
    }

    #[test]
    fn test_effect_value_box_reflect_differs_from_scalar_variant() {
        let boxed = EffectValue::BoxReflect(Box::new(42i32));
        let scalar = EffectValue::I32(42);
        assert_ne!(boxed, scalar, "BoxReflect 与标量变体必须是不同值");
    }

    // ===== Blackboard 容器（EffectBlackboard）=====

    fn blackboard_with(name: &str, value: EffectValue) -> EffectBlackboard {
        let mut bb = EffectBlackboard::default();
        bb.blackboard.insert(Name::new(name.to_string()), value);
        bb
    }

    #[test]
    fn blackboard_default_is_empty() {
        let bb = EffectBlackboard::default();
        assert!(bb.blackboard.is_empty());
    }

    #[test]
    fn blackboard_insert_then_get_roundtrip() {
        let bb = blackboard_with("hp", EffectValue::I32(10));
        let value = bb.blackboard.get(&Name::new("hp")).expect("插入后应可读取");
        assert_eq!(value.get::<&i32>(), Ok(&10));
    }

    #[test]
    fn blackboard_overwrite_replaces_previous() {
        let mut bb = EffectBlackboard::default();
        bb.blackboard.insert(Name::new("hp"), EffectValue::I32(10));
        bb.blackboard.insert(Name::new("hp"), EffectValue::I32(99));
        assert_eq!(
            bb.blackboard.get(&Name::new("hp")),
            Some(&EffectValue::I32(99))
        );
    }

    #[test]
    fn blackboard_remove_deletes_entry() {
        let mut bb = blackboard_with("hp", EffectValue::I32(10));
        bb.blackboard.remove(&Name::new("hp"));
        assert!(bb.blackboard.get(&Name::new("hp")).is_none());
        assert!(bb.blackboard.is_empty());
    }

    #[test]
    fn blackboard_missing_key_returns_none() {
        let bb = EffectBlackboard::default();
        assert!(bb.blackboard.get(&Name::new("missing")).is_none());
    }

    #[test]
    fn blackboard_type_mismatch_errors() {
        let bb = blackboard_with("hp", EffectValue::I32(10));
        let value = bb.blackboard.get(&Name::new("hp")).expect("插入后应可读取");
        let result: Result<&f32, _> = value.get::<&f32>();
        assert!(result.is_err(), "I32 值按 f32 读取必须报错");
    }

    // ===== 各变体 TryFrom（不可变 + 可变）全路径 =====

    #[test]
    fn try_from_all_signed_mut_variants() {
        let mut v = EffectValue::I8(8);
        *v.get_mut::<&mut i8>().expect("应可取出 i8 可变引用") = 9;
        assert_eq!(v.get::<&i8>(), Ok(&9));

        let mut v = EffectValue::I16(16);
        *v.get_mut::<&mut i16>().expect("应可取出 i16 可变引用") = 17;
        assert_eq!(v.get::<&i16>(), Ok(&17));

        let mut v = EffectValue::I64(64);
        *v.get_mut::<&mut i64>().expect("应可取出 i64 可变引用") = 65;
        assert_eq!(v.get::<&i64>(), Ok(&65));
    }

    #[test]
    fn try_from_all_unsigned_mut_variants() {
        let mut v = EffectValue::U8(1);
        *v.get_mut::<&mut u8>().expect("应可取出 u8 可变引用") = 2;
        assert_eq!(v.get::<&u8>(), Ok(&2));

        let mut v = EffectValue::U16(1);
        *v.get_mut::<&mut u16>().expect("应可取出 u16 可变引用") = 2;
        assert_eq!(v.get::<&u16>(), Ok(&2));

        let mut v = EffectValue::U32(1);
        *v.get_mut::<&mut u32>().expect("应可取出 u32 可变引用") = 2;
        assert_eq!(v.get::<&u32>(), Ok(&2));

        let mut v = EffectValue::U64(1);
        *v.get_mut::<&mut u64>().expect("应可取出 u64 可变引用") = 2;
        assert_eq!(v.get::<&u64>(), Ok(&2));
    }

    #[test]
    fn try_from_float_mut_variants() {
        let mut v = EffectValue::F32(1.0);
        *v.get_mut::<&mut f32>().expect("应可取出 f32 可变引用") = 2.0;
        assert_eq!(v.get::<&f32>(), Ok(&2.0));

        let mut v = EffectValue::F64(1.0);
        *v.get_mut::<&mut f64>().expect("应可取出 f64 可变引用") = 2.0;
        assert_eq!(v.get::<&f64>(), Ok(&2.0));
    }

    #[test]
    fn try_from_cow_mut_and_string_owned() {
        let mut v = EffectValue::String("hello".into());
        *v.get_mut::<&mut Cow<'static, str>>()
            .expect("应可取出 Cow 可变引用") = Cow::Owned("world".into());
        assert_eq!(v.get::<&Cow<'static, str>>(), Ok(&Cow::Borrowed("world")));

        let v = EffectValue::String("owned".into());
        let owned: Result<String, _> = (&v).try_into();
        assert_eq!(owned, Ok("owned".to_string()));
    }

    #[test]
    fn try_from_vec_entity_get_and_mut() {
        let e1 = Entity::from_bits(1);
        let e2 = Entity::from_bits(2);
        let mut v = EffectValue::VecEntity(vec![e1, e2]);

        assert_eq!(v.get::<&Vec<Entity>>(), Ok(&vec![e1, e2]));

        v.get_mut::<&mut Vec<Entity>>()
            .expect("应可取出 Vec<Entity> 可变引用")
            .push(Entity::from_bits(3));
        assert_eq!(
            v.get::<&Vec<Entity>>(),
            Ok(&vec![e1, e2, Entity::from_bits(3)])
        );
    }

    #[test]
    fn try_from_box_reflect_get_and_mut() {
        let mut v = EffectValue::BoxReflect(Box::new(42i32));

        {
            let b = v
                .get_mut::<&mut Box<dyn Reflect>>()
                .expect("应可取出 BoxReflect 可变引用");
            assert!(b.as_ref().reflect_partial_eq(&42i32).unwrap_or(false));
        }

        let b = v
            .get::<&Box<dyn Reflect>>()
            .expect("应可取出 BoxReflect 引用");
        assert!(b.as_ref().reflect_partial_eq(&42i32).unwrap_or(false));
    }

    #[test]
    fn try_from_wrong_variant_errors_for_mut() {
        let mut v = EffectValue::String("x".into());
        assert!(v.get_mut::<&mut i32>().is_err());
    }

    // ===== Clone / PartialEq 未覆盖变体 =====

    #[test]
    fn clone_preserves_vec_entity_and_string() {
        let vec = EffectValue::VecEntity(vec![Entity::from_bits(1)]);
        assert_eq!(vec.clone(), vec);

        let string = EffectValue::String("copy".into());
        assert_eq!(string.clone(), string);

        let entity = EffectValue::Entity(Entity::from_bits(5));
        assert_eq!(entity.clone(), entity);
    }

    #[test]
    fn partial_eq_mismatched_variants_are_not_equal() {
        assert_ne!(EffectValue::I32(1), EffectValue::F32(1.0));
        assert_ne!(
            EffectValue::Entity(Entity::from_bits(1)),
            EffectValue::I32(1)
        );
        assert_ne!(
            EffectValue::VecEntity(vec![]),
            EffectValue::String("".into())
        );
        assert_ne!(EffectValue::U64(1), EffectValue::I64(1));
    }

    #[test]
    fn partial_eq_matching_scalar_variants_are_equal() {
        assert_eq!(EffectValue::U16(7), EffectValue::U16(7));
        assert_eq!(EffectValue::F64(1.5), EffectValue::F64(1.5));
        assert_eq!(
            EffectValue::Entity(Entity::from_bits(9)),
            EffectValue::Entity(Entity::from_bits(9))
        );
    }
}
