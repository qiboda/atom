use std::borrow::Cow;

use bevy::{prelude::Entity, reflect::Reflect};

#[allow(unused)]
#[derive(Debug)]
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

    String(Cow<'static, str>),

    Vec(Vec<EffectValue>),

    BoxReflect(Box<dyn Reflect>),
}

trait BlackBoardValue {
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

impl<'a> TryFrom<&'a EffectValue> for &'a Vec<EffectValue> {
    type Error = &'static str;

    fn try_from(value: &'a EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::Vec(v) => Ok(v),
            _ => Err("not Vec"),
        }
    }
}

impl<'a> TryFrom<&'a mut EffectValue> for &'a mut Vec<EffectValue> {
    type Error = &'static str;

    fn try_from(value: &'a mut EffectValue) -> Result<Self, Self::Error> {
        match value {
            EffectValue::Vec(v) => Ok(v),
            _ => Err("not Vec"),
        }
    }
}

// impl<'a, T> TryFrom<&'a EffectValue> for &'a Vec<T> {
//     type Error = &'static str;
//
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
//

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

// test
#[cfg(test)]
mod test {
    use std::borrow::Cow;

    use bevy::reflect::Reflect;

    use crate::graph::blackboard::BlackBoardValue;

    use super::EffectValue;

    #[test]
    fn black_board_value() {
        let bo = Box::new(32);
        let br = EffectValue::BoxReflect(bo);
        if let EffectValue::BoxReflect(v) = br {
            assert_eq!(32, *v.downcast_ref::<i32>().unwrap());
        }
    }

    #[test]
    fn black_board_value_try_from() {
        let br_i32 = EffectValue::I32(100);
        assert!((&br_i32).try_into() == Ok(&100i32));

        let br_vec = EffectValue::Vec(vec![EffectValue::I32(100)]);
        if let Ok(vec_value) = TryInto::<&Vec<EffectValue>>::try_into(&br_vec) {
            for elem in vec_value {
                assert!(elem.try_into() == Ok(&100));
            }
        }

        let br_str = EffectValue::String("laksdjfk".into());
        assert!((&br_str).try_into() == Ok(&Cow::<'static, str>::Owned("laksdjfk".into())));

        let br_box = EffectValue::BoxReflect(Box::new(vec![32]));
        let v = TryInto::<&Box<dyn Reflect>>::try_into(&br_box);
        if let Ok(v) = v {
            assert!(v.downcast_ref::<Vec<i32>>() == Some(&Box::new(vec![32])));
        }
    }

    #[test]
    fn black_boardrvalue_get() {
        let br_i32 = EffectValue::I32(100);
        assert!(br_i32.get() == Ok(&100i32));

        let br_str = EffectValue::String("laksdjfk".into());
        assert!(br_str.get() == Ok(&Cow::<'static, str>::Owned("laksdjfk".into())));

        let br_box = EffectValue::BoxReflect(Box::new(vec![32]));
        let v = br_box.get::<&Box<dyn Reflect>>();
        if let Ok(v) = v {
            assert!(v.downcast_ref::<Vec<i32>>() == Some(&Box::new(vec![32])));
        }

        let mut br_i32 = EffectValue::I32(100);
        assert!(br_i32.get_mut() == Ok(&mut 100i32));
        *br_i32.get_mut::<&mut i32>().unwrap() = 200;
        assert!(br_i32.get() == Ok(&200i32));

        let br_str = EffectValue::String("laksdjfk".into());
        assert!(br_str.get() == Ok(&Cow::<'static, str>::Owned("laksdjfk".into())));

        let br_box = EffectValue::BoxReflect(Box::new(vec![32]));
        let v = br_box.get::<&Box<dyn Reflect>>();
        if let Ok(v) = v {
            assert!(v.downcast_ref::<Vec<i32>>() == Some(&Box::new(vec![32])));
        }
    }
}
