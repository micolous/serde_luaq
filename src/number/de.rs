//! Deserialise to a [LuaNumber] field.

use crate::{Error, LuaNumber};
use serde::{de::Visitor, forward_to_deserialize_any, Deserializer};

macro_rules! number_visitor {
    ($meth:ident $typ:ty) => {
        #[inline]
        fn $meth<E>(self, v: $typ) -> Result<Self::Value, E> {
            Ok(v.into())
        }
    };
}

impl<'de> serde::Deserialize<'de> for LuaNumber {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LuaNumberVisitor;

        impl<'de> Visitor<'de> for LuaNumberVisitor {
            type Value = LuaNumber;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("any valid Lua number")
            }

            number_visitor!(visit_f32 f32);
            number_visitor!(visit_f64 f64);
            number_visitor!(visit_i8 i8);
            number_visitor!(visit_i16 i16);
            number_visitor!(visit_i32 i32);
            number_visitor!(visit_i64 i64);
            number_visitor!(visit_u8 u8);
            number_visitor!(visit_u16 u16);
            number_visitor!(visit_u32 u32);
        }

        deserializer.deserialize_any(LuaNumberVisitor {})
    }
}

macro_rules! deserialize_number {
    ($method:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
        {
            match self {
                LuaNumber::Integer(n) => visitor.visit_i64(n),
                LuaNumber::Float(n) => visitor.visit_f64(n),
            }
        }
    };
}

impl<'de> serde::Deserializer<'de> for LuaNumber {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            LuaNumber::Integer(v) => visitor.visit_i64(v),
            LuaNumber::Float(v) => visitor.visit_f64(v),
        }
    }

    deserialize_number!(deserialize_i8);
    deserialize_number!(deserialize_i16);
    deserialize_number!(deserialize_i32);
    deserialize_number!(deserialize_i64);
    deserialize_number!(deserialize_i128);
    deserialize_number!(deserialize_u8);
    deserialize_number!(deserialize_u16);
    deserialize_number!(deserialize_u32);
    deserialize_number!(deserialize_u64);
    deserialize_number!(deserialize_u128);
    deserialize_number!(deserialize_f32);
    deserialize_number!(deserialize_f64);

    forward_to_deserialize_any! {
        bool char str string enum ignored_any
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct identifier
    }
}
