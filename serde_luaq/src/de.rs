//! Deserializes a [`LuaValue`] using Serde.

use crate::{
    lua_value, return_statement, script,
    value::{from_utf8_cow, to_utf8_cow},
    Error, LuaNumber, LuaTableEntry, LuaValue,
};
use serde::{
    de::{
        self, DeserializeSeed, EnumAccess, Expected, IntoDeserializer, MapAccess, SeqAccess,
        Unexpected, VariantAccess, Visitor,
    },
    forward_to_deserialize_any, Deserialize, Deserializer,
};
use std::{borrow::Cow, collections::BTreeMap, vec};

fn utf8_str<E: serde::de::Error>(v: Cow<'_, [u8]>) -> Result<Cow<'_, str>, E> {
    from_utf8_cow(v)
        .map_err(|(_, b)| serde::de::Error::invalid_value(Unexpected::Bytes(&b), &"UTF8 string"))
}

fn visit_array<'de, V>(array: Vec<LuaTableEntry<'de>>, visitor: V) -> Result<V::Value, Error>
where
    V: Visitor<'de>,
{
    let len = array.len();
    let mut deserializer = SeqDeserializer::new(array)?;
    let seq = visitor.visit_seq(&mut deserializer)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(seq)
    } else {
        Err(serde::de::Error::invalid_length(
            len,
            &"fewer elements in array",
        ))
    }
}

macro_rules! deserialize_value_number {
    ($method:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
        {
            match self {
                LuaValue::Number(n) => n.$method(visitor),
                _ => Err(self.invalid_type(&visitor)),
            }
        }
    };
}

impl<'de> serde::Deserializer<'de> for LuaValue<'de> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            LuaValue::Nil => visitor.visit_none(),
            LuaValue::Boolean(v) => visitor.visit_bool(v),
            LuaValue::Number(v) => v.deserialize_any(visitor),
            LuaValue::String(v) => match v {
                Cow::Borrowed(b) => visitor.visit_borrowed_bytes(b),
                Cow::Owned(b) => visitor.visit_byte_buf(b),
            },
            LuaValue::Table(v) => LuaTableWrapper(v.into_iter()).deserialize_seq(visitor),
        }
    }

    deserialize_value_number!(deserialize_i8);
    deserialize_value_number!(deserialize_i16);
    deserialize_value_number!(deserialize_i32);
    deserialize_value_number!(deserialize_i64);
    deserialize_value_number!(deserialize_i128);
    deserialize_value_number!(deserialize_u8);
    deserialize_value_number!(deserialize_u16);
    deserialize_value_number!(deserialize_u32);
    deserialize_value_number!(deserialize_u64);
    deserialize_value_number!(deserialize_u128);
    deserialize_value_number!(deserialize_f32);
    deserialize_value_number!(deserialize_f64);

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            LuaValue::Nil => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            LuaValue::Table(value) => {
                LuaTableWrapper(value.into_iter()).deserialize_enum(name, variants, visitor)
            }
            LuaValue::String(variant) => visitor.visit_enum(EnumDeserializer {
                variant,
                value: None,
            }),
            other => Err(serde::de::Error::invalid_type(
                other.unexpected(),
                &"string or map",
            )),
        }
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let _ = name;
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            LuaValue::Boolean(v) => visitor.visit_bool(v),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            // #[cfg(any(feature = "std", feature = "alloc"))]
            LuaValue::String(v) => match utf8_str::<Error>(v)? {
                Cow::Borrowed(v) => visitor.visit_borrowed_str(v),
                Cow::Owned(v) => visitor.visit_string(v),
            },
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            // #[cfg(any(feature = "std", feature = "alloc"))]
            LuaValue::String(v) => visitor.visit_bytes(&v),
            LuaValue::Table(v) => visit_array(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            LuaValue::Nil => visitor.visit_unit(),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            LuaValue::Table(v) => visit_array(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            LuaValue::Table(v) => LuaTableWrapper(v.into_iter()).deserialize_any(visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            LuaValue::Table(v) => LuaTableWrapper(v.into_iter()).deserialize_any(visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        drop(self);
        visitor.visit_unit()
    }
}

struct EnumDeserializer<'a> {
    variant: Cow<'a, [u8]>,
    value: Option<LuaValue<'a>>,
}

impl<'de> EnumAccess<'de> for EnumDeserializer<'de> {
    type Error = Error;
    type Variant = VariantDeserializer<'de>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Error>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = self.variant.into_deserializer();
        let visitor = VariantDeserializer { value: self.value };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}

impl<'de> IntoDeserializer<'de, Error> for LuaValue<'de> {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

// impl<'de> IntoDeserializer<'de, Error> for &'de LuaValue<'de> {
//     type Deserializer = Self;

//     fn into_deserializer(self) -> Self::Deserializer {
//         self
//     }
// }

struct VariantDeserializer<'a> {
    value: Option<LuaValue<'a>>,
}

impl<'de> VariantAccess<'de> for VariantDeserializer<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        match self.value {
            Some(value) => Deserialize::deserialize(value),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"newtype variant",
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(LuaValue::Table(v)) => {
                if v.is_empty() {
                    visitor.visit_unit()
                } else {
                    visit_array(v, visitor)
                }
            }
            Some(other) => Err(serde::de::Error::invalid_type(
                other.unexpected(),
                &"tuple variant",
            )),
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"tuple variant",
            )),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(LuaValue::Table(v)) => LuaTableWrapper(v.into_iter()).deserialize_any(visitor),
            Some(other) => Err(serde::de::Error::invalid_type(
                other.unexpected(),
                &"struct variant",
            )),
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"struct variant",
            )),
        }
    }
}

impl LuaValue<'_> {
    #[cold]
    fn invalid_type<E>(&self, exp: &dyn Expected) -> E
    where
        E: serde::de::Error,
    {
        serde::de::Error::invalid_type(self.unexpected(), exp)
    }

    #[cold]
    fn unexpected(&self) -> Unexpected<'_> {
        match self {
            LuaValue::Nil => Unexpected::Unit,
            LuaValue::Boolean(b) => Unexpected::Bool(*b),
            LuaValue::Number(LuaNumber::Integer(n)) => Unexpected::Signed(*n),
            LuaValue::Number(LuaNumber::Float(n)) => Unexpected::Float(*n),
            LuaValue::String(s) => Unexpected::Bytes(s),
            LuaValue::Table(_) => Unexpected::Map,
        }
    }
}

impl LuaTableEntry<'_> {
    #[cold]
    fn invalid_type<E>(&self, exp: &dyn Expected) -> E
    where
        E: serde::de::Error,
    {
        serde::de::Error::invalid_type(self.unexpected(), exp)
    }

    #[cold]
    fn unexpected(&self) -> Unexpected<'_> {
        match self {
            LuaTableEntry::NameValue(_, _) | LuaTableEntry::KeyValue(_, _) => Unexpected::Map,
            LuaTableEntry::Value(_) => Unexpected::Seq,
        }
    }
}

impl MapKeyDeserializer<'_> {
    #[cold]
    fn unexpected_key(&self) -> Unexpected<'_> {
        match self {
            Self::NameValue(k) => Unexpected::Str(k),
            Self::KeyValue(k) => k.unexpected(),
            Self::Value(k) => Unexpected::Signed(*k),
        }
    }
}

struct SeqDeserializer<'a> {
    iter: vec::IntoIter<LuaValue<'a>>,
}

impl<'a> SeqDeserializer<'a> {
    /// Create a new sequence deserializer.
    fn new(vec: Vec<LuaTableEntry<'a>>) -> Result<Self, Error> {
        // Check to see if we need to re-number things
        let mut has_keys = false;
        for entry in vec.iter() {
            if !matches!(entry, LuaTableEntry::Value(_)) {
                if !matches!(
                    entry,
                    LuaTableEntry::KeyValue(LuaValue::Number(LuaNumber::Integer(_)), _)
                ) {
                    return Err(entry.invalid_type(&"Table with integer or implicit keys"));
                }

                // At least one entry with an explicit integer key.
                has_keys = true;
                break;
            }
        }

        if !has_keys {
            // We can extract all the entries directly.
            let vec: Vec<LuaValue<'a>> = vec.into_iter().map(|e| e.move_value()).collect();

            return Ok(SeqDeserializer {
                iter: vec.into_iter(),
            });
        }

        // Scan over the entire Vec, and overwrite entries.
        let mut h = BTreeMap::new();
        let mut i = 1;
        let mut highest_key = 0;
        for entry in vec {
            match entry {
                LuaTableEntry::KeyValue(LuaValue::Number(LuaNumber::Integer(key)), value) => {
                    h.insert(key, value);
                    highest_key = highest_key.max(key);
                }
                LuaTableEntry::Value(value) => {
                    h.insert(i, value);
                    i += 1;
                    highest_key = highest_key.max(i);
                }
                _ => unreachable!(),
            }
        }

        // Convert to a Vec with no gaps, with keys starting at 1.
        let mut vec = Vec::with_capacity((highest_key + 1) as usize);
        let mut next_key = 1;
        for (k, v) in h {
            if k > next_key {
                for _ in next_key..k {
                    // Fill empty entries with nil
                    vec.push(LuaValue::Nil);
                }
            }

            vec.push(v);
            next_key = k + 1;
        }

        Ok(SeqDeserializer {
            iter: vec.into_iter(),
        })
    }
}

impl<'de> SeqAccess<'de> for SeqDeserializer<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

struct MapDeserializer<'a, T>
where
    T: Iterator<Item = LuaTableEntry<'a>>,
{
    // iter: <Vec<LuaTableEntry<'a>> as IntoIterator>::IntoIter,
    iter: T,
    value: Option<LuaValue<'a>>,
    next_numeric_index: i64,
}

impl<'a, T> MapDeserializer<'a, T>
where
    T: Iterator<Item = LuaTableEntry<'a>>,
{
    fn new(iter: T) -> Self {
        MapDeserializer {
            iter,
            value: None,
            next_numeric_index: 1,
        }
    }
}

impl<'de, T> MapAccess<'de> for MapDeserializer<'de, T>
where
    T: Iterator<Item = LuaTableEntry<'de>>,
{
    type Error = Error;

    fn next_key_seed<S>(&mut self, seed: S) -> Result<Option<S::Value>, Error>
    where
        S: DeserializeSeed<'de>,
    {
        // Copy the entry without a value and pass to MapKeyDeserializer
        match self.iter.next() {
            Some(LuaTableEntry::KeyValue(key, value)) => {
                self.value = Some(value);

                let key_de = MapKeyDeserializer::KeyValue(key);
                seed.deserialize(key_de).map(Some)
            }
            Some(LuaTableEntry::NameValue(key, value)) => {
                self.value = Some(value);
                let key_de = MapKeyDeserializer::NameValue(key);
                seed.deserialize(key_de).map(Some)
            }
            Some(LuaTableEntry::Value(value)) => {
                self.value = Some(value);
                let key_de = MapKeyDeserializer::Value(self.next_numeric_index);
                self.next_numeric_index += 1;
                seed.deserialize(key_de).map(Some)
            }

            None => Ok(None),
        }
    }

    fn next_value_seed<S>(&mut self, seed: S) -> Result<S::Value, Error>
    where
        S: DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::custom("value is missing")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

enum MapKeyDeserializer<'de> {
    KeyValue(LuaValue<'de>),
    NameValue(Cow<'de, str>),
    Value(i64),
}

macro_rules! deserialize_numeric_key {
    ($method:ident) => {
        deserialize_numeric_key!($method, deserialize_number);
    };

    ($method:ident, $using:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
        {
            match self {
                Self::KeyValue(LuaValue::Number(key)) => key.$method(visitor),
                Self::Value(key) => visitor.visit_i64(key),
                key => Err(serde::de::Error::invalid_type(
                    key.unexpected_key(),
                    &visitor,
                )),
            }
        }
    };
}

impl<'de> serde::Deserializer<'de> for MapKeyDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::KeyValue(key) => key.deserialize_any(visitor),
            Self::NameValue(key) => match key {
                Cow::Borrowed(key) => visitor.visit_borrowed_str(key),
                Cow::Owned(key) => visitor.visit_string(key),
            },
            Self::Value(key) => visitor.visit_i64(key),
        }
    }

    deserialize_numeric_key!(deserialize_i8);
    deserialize_numeric_key!(deserialize_i16);
    deserialize_numeric_key!(deserialize_i32);
    deserialize_numeric_key!(deserialize_i64);
    deserialize_numeric_key!(deserialize_u8);
    deserialize_numeric_key!(deserialize_u16);
    deserialize_numeric_key!(deserialize_u32);
    deserialize_numeric_key!(deserialize_u64);
    deserialize_numeric_key!(deserialize_f32);
    deserialize_numeric_key!(deserialize_f64);

    // #[cfg(feature = "float_roundtrip")]
    // deserialize_numeric_key!(deserialize_f32, do_deserialize_f32);
    deserialize_numeric_key!(deserialize_i128, do_deserialize_i128);
    deserialize_numeric_key!(deserialize_u128, do_deserialize_u128);

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::KeyValue(LuaValue::Boolean(key)) => visitor.visit_bool(key),
            key => Err(serde::de::Error::invalid_type(
                key.unexpected_key(),
                &visitor,
            )),
        }
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        // Map keys cannot be nil.
        visitor.visit_some(self)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::KeyValue(key) => key
                .into_deserializer()
                .deserialize_enum(name, variants, visitor),
            Self::NameValue(variant) => visitor.visit_enum(EnumDeserializer {
                variant: to_utf8_cow(variant),
                value: None,
            }),
            Self::Value(key) => visitor.visit_enum(EnumDeserializer {
                variant: key.to_string().into_bytes().into(),
                value: None,
            }),
        }
    }

    forward_to_deserialize_any! {
        char str string bytes byte_buf unit unit_struct seq tuple tuple_struct
        map struct identifier ignored_any
    }
}

/// Internal wrapper for [`Vec<LuaTableEntry>`] that we can implement
/// [`serde::Deserializer`] on.
struct LuaTableWrapper<'a, T>(T)
where
    T: Iterator<Item = LuaTableEntry<'a>> + ExactSizeIterator;

impl<'de, T> serde::Deserializer<'de> for LuaTableWrapper<'de, T>
where
    T: Iterator<Item = LuaTableEntry<'de>> + ExactSizeIterator,
{
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let len = self.0.len();
        let mut deserializer: MapDeserializer<'_, T> = MapDeserializer::new(self.0);
        let map = visitor.visit_map(&mut deserializer)?;
        let remaining = deserializer.iter.len();
        if remaining == 0 {
            Ok(map)
        } else {
            Err(serde::de::Error::invalid_length(
                len,
                &"fewer elements in map",
            ))
        }
    }

    fn deserialize_enum<V>(
        mut self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (variant, value) = match self.0.next() {
            Some(LuaTableEntry::KeyValue(LuaValue::String(k), v)) => (k, v),
            Some(LuaTableEntry::NameValue(k, v)) => (to_utf8_cow(k), v),
            _ => {
                return Err(serde::de::Error::invalid_value(
                    Unexpected::Map,
                    &"map with a single key",
                ));
            }
        };

        // enums are encoded in json as maps with a single key:value pair
        if self.0.next().is_some() {
            return Err(serde::de::Error::invalid_value(
                Unexpected::Map,
                &"map with a single key",
            ));
        }

        visitor.visit_enum(EnumDeserializer {
            variant,
            value: Some(value),
        })
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        drop(self);
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct identifier
    }
}

/// The format of the input Lua buffer.
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum LuaFormat {
    /// A bare Lua expression:
    ///
    /// ```lua
    /// {hello = "world"}
    /// ```
    #[default]
    Value,

    /// A Lua script containing only variable assignments:
    ///
    /// ```lua
    /// hello = "world"
    /// ```
    Script,

    /// A Lua `return` statement:
    ///
    /// ```lua
    /// return {hello = "world"}
    /// ```
    Return,
}

/// Parses a byte slice containing a Lua expression in [`format`][LuaFormat].
///
/// The Lua expression may only consist of simple data, with restrictions similar to JSON.
///
/// For more details about type mapping rules and parameters,
/// [see the crate docs][crate#data-types].
///
/// [serde-num-keys]: https://github.com/serde-rs/serde/issues/2358
/// [surrogate]: https://www.unicode.org/versions/Unicode17.0.0/core-spec/chapter-3/#G2630
/// [RFC 2279]: https://www.rfc-editor.org/rfc/rfc2279
/// [RFC 3629]: https://www.rfc-editor.org/rfc/rfc3629
pub fn from_slice<'a, T>(b: &'a [u8], format: LuaFormat, max_depth: u16) -> Result<T, Error>
where
    T: de::Deserialize<'a>,
{
    let v = match format {
        LuaFormat::Value => lua_value(b, max_depth)?,
        LuaFormat::Script => script(b, max_depth)?.into_iter().collect(),
        LuaFormat::Return => return_statement(b, max_depth)?,
    };

    Deserialize::deserialize(v)
}

/// Parses a [`str`] containing a Lua expression in [`format`][LuaFormat].
///
/// See [`from_slice()`] for more details.
///
/// ## Warning
///
/// [Lua is "8-bit clean"][lua2.1]: its strings (and source files) may contain any 8-bit value,
/// including null bytes (`\0`), and is _encoding agnostic_ - equivalent to `[u8]` in Rust.
///
/// This method assumes that a Lua expression is encoded as valid RFC 3629 UTF-8.
///
/// [lua2.1]: https://www.lua.org/manual/5.4/manual.html#2.1
#[inline]
pub fn from_str<'a, T>(b: &'a str, format: LuaFormat, max_depth: u16) -> Result<T, Error>
where
    T: de::Deserialize<'a>,
{
    from_slice(b.as_bytes(), format, max_depth)
}
