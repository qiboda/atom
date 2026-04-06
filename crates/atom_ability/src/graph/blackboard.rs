use bevy::{platform::collections::HashMap, prelude::*};
use std::borrow::Cow;

use bevy::{prelude::Entity, reflect::Reflect};

#[derive(Debug, Component, Default)]
pub struct EffectBlackboard {
    pub blackboard: HashMap<Name, EffectValue>,
}

#[allow(unused)]
#[derive(Debug, Reflect, PartialEq, Clone)]
pub enum EffectValue {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),

    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),

    F32(f32),
    F64(f64),

    Entity(Entity),
    VecEntity(Vec<Entity>),

    String(Cow<'static, str>),
    // Vec(Vec<EffectValue>),
    // TODO: add when bevy support
    // BoxReflect(Box<dyn Reflect>),
}

pub trait BlackBoardValue {
    fn get<'a, T>(&'a self) -> Result<T, T::Error>
    where
        T: TryFrom<&'a Self>;

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

// impl<'a> TryFrom<&'a EffectValue> for &'a Box<dyn Reflect> {
//     type Error = &'static str;

//     fn try_from(value: &'a EffectValue) -> Result<Self, Self::Error> {
//         match value {
//             EffectValue::BoxReflect(v) => Ok(v),
//             _ => Err("not BoxReflect"),
//         }
//     }
// }

// impl<'a> TryFrom<&'a mut EffectValue> for &'a mut Box<dyn Reflect> {
//     type Error = &'static str;

//     fn try_from(value: &'a mut EffectValue) -> Result<Self, Self::Error> {
//         match value {
//             EffectValue::BoxReflect(v) => Ok(v),
//             _ => Err("not BoxReflect"),
//         }
//     }
// }

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
}
