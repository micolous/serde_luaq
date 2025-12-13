use crate::{
    valid_lua_identifier,
    value::{from_utf8_cow, to_utf8_cow},
    LuaNumber, LuaValue,
};
use std::{borrow::Cow, str::from_utf8};

/// Lua table entry.
///
/// Reference: <https://www.lua.org/manual/5.4/manual.html#3.4.9>
#[derive(Debug, Clone, PartialEq)]
pub enum LuaTableEntry<'a> {
    /// Table entry in the form: `["foo"] = "bar"` or `[123] = "bar"`
    KeyValue(Box<(LuaValue<'a>, LuaValue<'a>)>),

    /// Table entry in the form: `foo = "bar"`
    ///
    /// > A field of the form `name = exp` is equivalent to `["name"] = exp`.
    ///
    /// Also:
    ///
    /// > Names (also called identifiers) in Lua can be any string of Latin
    /// > letters, Arabic-Indic digits, and underscores, not beginning with a
    /// > digit and not being a reserved word.
    ///
    /// This is represented as a `str`, as these are valid RFC 3629 UTF-8 on
    /// Lua 5.2 and later with default build settings (`LUA_UCID = 0`).
    NameValue(Box<(Cow<'a, str>, LuaValue<'a>)>),

    /// Bare table entry without a key: `"bar"`
    ///
    /// > fields of the form `exp` are equivalent to `[i] = exp`, where `i`
    /// > are consecutive numerical integers, starting with 1. Fields in the
    /// > other formats do not affect this counting.
    Value(Box<LuaValue<'a>>),
}

impl<'a> LuaTableEntry<'a> {
    /// Get the key of the table entry.
    ///
    /// This will clone the key of [`KeyValue`][LuaTableEntry::KeyValue] entries.
    ///
    /// Returns [`None`] for [`Value`][LuaTableEntry::Value] entries.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use serde_luaq::{LuaValue, LuaTableEntry};
    ///
    /// assert_eq!(Some(LuaValue::integer(1)), LuaTableEntry::KeyValue(LuaValue::integer(1), LuaValue::Boolean(true)).key());
    /// assert_eq!(Some(b"foo".into()), LuaTableEntry::NameValue("foo".into(), LuaValue::Boolean(true)).key());
    /// assert_eq!(None, LuaTableEntry::Value(LuaValue::Boolean(true)).key());
    /// ```
    pub fn key(&'a self) -> Option<LuaValue<'a>> {
        match self {
            LuaTableEntry::KeyValue(b) => Some(b.0.clone()),
            LuaTableEntry::NameValue(b) => Some(LuaValue::String(to_utf8_cow(b.0.clone()))),
            LuaTableEntry::Value(_) => None,
        }
    }

    /// Get the value of the table entry.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use serde_luaq::{LuaValue, LuaTableEntry};
    ///
    /// assert_eq!(&LuaValue::Boolean(true), LuaTableEntry::KeyValue(LuaValue::integer(1), LuaValue::Boolean(true)).value());
    /// assert_eq!(&LuaValue::Boolean(true), LuaTableEntry::NameValue("foo".into(), LuaValue::Boolean(true)).value());
    /// assert_eq!(&LuaValue::Boolean(true), LuaTableEntry::Value(LuaValue::Boolean(true)).value());
    /// ```
    pub fn value(&'a self) -> &'a LuaValue<'a> {
        match self {
            LuaTableEntry::KeyValue(b) => &b.1,
            LuaTableEntry::NameValue(b) => &b.1,
            LuaTableEntry::Value(value) => value,
        }
    }

    /// Move the value out of the table entry.
    pub fn move_value(self) -> LuaValue<'a> {
        match self {
            LuaTableEntry::KeyValue(b) => b.1,
            LuaTableEntry::NameValue(b) => b.1,
            LuaTableEntry::Value(value) => *value,
        }
    }
}

impl<'a> TryFrom<LuaTableEntry<'a>> for (Cow<'a, [u8]>, LuaValue<'a>) {
    type Error = LuaTableEntry<'a>;

    /// Converts a [`LuaTableEntry::KeyValue`] (with [`LuaValue::String`]) and
    /// [`LuaTableEntry::NameValue`] into `(Cow<'a, u8>, LuaValue)`.
    ///
    /// Returns `Err` for other types.
    ///
    /// This is intended to help convert an `Iterator<Item = LuaTableEntry>`
    /// into a `HashMap<&[u8], LuaValue>`.
    fn try_from(value: LuaTableEntry<'a>) -> Result<Self, Self::Error> {
        match value {
            LuaTableEntry::KeyValue(b) if matches!(&b.0, LuaValue::String(_)) => {
                let LuaValue::String(k) = b.0 else {
                    unreachable!();
                };
                Ok((k, b.1))
            }
            LuaTableEntry::NameValue(b) => Ok((to_utf8_cow(b.0), b.1)),
            other => Err(other),
        }
    }
}

impl<'a> TryFrom<LuaTableEntry<'a>> for (Cow<'a, str>, LuaValue<'a>) {
    type Error = LuaTableEntry<'a>;

    /// Converts [`KeyValue`][] (with [`String`][] key) or [`NameValue`][LuaTableEntry::NameValue] into
    /// `(Cow<'a str>, LuaValue<'a>)`.
    ///
    /// Returns [`Err`] for other types, and [`KeyValue`][] keys which are not valid UTF-8.
    ///
    /// [`KeyValue`]: LuaTableEntry::KeyValue
    /// [`String`]: LuaValue::String
    fn try_from(value: LuaTableEntry<'a>) -> Result<Self, Self::Error> {
        match value {
            LuaTableEntry::KeyValue(b) if matches!(&b.0, LuaValue::String(_)) => {
                let LuaValue::String(k) = b.0 else {
                    unreachable!();
                };
                match from_utf8_cow(k) {
                    Ok(k) => Ok((k, b.1)),

                    // Error, return the original value back
                    Err((_, k)) => Err(LuaTableEntry::KeyValue(Box::new((
                        LuaValue::String(k),
                        b.1,
                    )))),
                }
            }
            LuaTableEntry::NameValue(b) => Ok(*b),
            other => Err(other),
        }
    }
}

impl<'a> From<(&'a [u8], LuaValue<'a>)> for LuaTableEntry<'a> {
    /// Converts a [`&[u8]`][u8] key and [`LuaValue`] into a [`LuaTableEntry::KeyValue`] with
    /// [`LuaValue::String`] key.
    ///
    /// If the key can be used as a valid Lua identifier, the value is instead
    /// represented as a [`LuaTableEntry::NameValue`].
    fn from((k, v): (&'a [u8], LuaValue<'a>)) -> Self {
        if valid_lua_identifier(k) {
            Self::NameValue(Box::new((from_utf8(k).unwrap().into(), v)))
        } else {
            Self::KeyValue(Box::new((LuaValue::String(k.into()), v)))
        }
    }
}

impl<'a, const N: usize> From<(&'a [u8; N], LuaValue<'a>)> for LuaTableEntry<'a> {
    /// Converts a [`&[u8; N]`][u8] key and [`LuaValue`] into a
    /// [`LuaTableEntry::KeyValue`] with [`LuaValue::String`] key.
    ///
    /// If the key can be used as a valid Lua identifier, the value is instead
    /// represented as a [`LuaTableEntry::NameValue`].
    fn from((k, v): (&'a [u8; N], LuaValue<'a>)) -> Self {
        if valid_lua_identifier(k) {
            Self::NameValue(Box::new((from_utf8(k).unwrap().into(), v)))
        } else {
            Self::KeyValue(Box::new((LuaValue::String(k.into()), v)))
        }
    }
}

impl<'a> From<(Vec<u8>, LuaValue<'a>)> for LuaTableEntry<'a> {
    /// Converts a [`Vec<u8>`] key and [`LuaValue`] into a
    /// [`LuaTableEntry::KeyValue`] with [`LuaValue::String`] key.
    ///
    /// If the key can be used as a valid Lua identifier, the value is instead
    /// represented as a [`LuaTableEntry::NameValue`].
    fn from((k, v): (Vec<u8>, LuaValue<'a>)) -> Self {
        if valid_lua_identifier(&k) {
            Self::NameValue(Box::new((String::from_utf8(k).unwrap().into(), v)))
        } else {
            Self::KeyValue(Box::new((LuaValue::String(k.into()), v)))
        }
    }
}

impl<'a> From<(Cow<'a, [u8]>, LuaValue<'a>)> for LuaTableEntry<'a> {
    /// Converts a `Cow<'a, [u8]>` key and [`LuaValue`] into a
    /// [`LuaTableEntry::KeyValue`] with [`LuaValue::String`] key.
    ///
    /// If the key can be used as a valid Lua identifier, the value is instead
    /// represented as a [`LuaTableEntry::NameValue`].
    fn from((k, v): (Cow<'a, [u8]>, LuaValue<'a>)) -> Self {
        if valid_lua_identifier(&k) {
            Self::NameValue(Box::new((from_utf8_cow(k).unwrap(), v)))
        } else {
            Self::KeyValue(Box::new((LuaValue::String(k), v)))
        }
    }
}

impl<'a> From<(&'a str, LuaValue<'a>)> for LuaTableEntry<'a> {
    /// Converts a [`str`] key and [`LuaValue`] into a
    /// [`LuaTableEntry::KeyValue`] with [`LuaValue::String`].
    ///
    /// If the key is a valid Lua identifier, the value is instead
    /// represented as a [`LuaTableEntry::NameValue`].
    fn from((k, v): (&'a str, LuaValue<'a>)) -> Self {
        if valid_lua_identifier(k.as_bytes()) {
            Self::NameValue(Box::new((k.into(), v)))
        } else {
            Self::KeyValue(Box::new((LuaValue::String(k.as_bytes().into()), v)))
        }
    }
}

impl<'a> From<(String, LuaValue<'a>)> for LuaTableEntry<'a> {
    /// Converts a [`String`] key and [`LuaValue`] into a
    /// [`LuaTableEntry::KeyValue`] with [`LuaValue::String`].
    ///
    /// If the key is a valid Lua identifier, the value is instead
    /// represented as a [`LuaTableEntry::NameValue`].
    fn from((k, v): (String, LuaValue<'a>)) -> Self {
        if valid_lua_identifier(k.as_bytes()) {
            Self::NameValue(Box::new((k.into(), v)))
        } else {
            Self::KeyValue(Box::new((LuaValue::String(k.into_bytes().into()), v)))
        }
    }
}

impl<'a> From<(Cow<'a, str>, LuaValue<'a>)> for LuaTableEntry<'a> {
    /// Converts a `Cow<'a, str>` key and [`LuaValue`] into a
    /// [`LuaTableEntry::KeyValue`] with [`LuaValue::String`].
    ///
    /// If the key is a valid Lua identifier, the value is instead
    /// represented as a [`LuaTableEntry::NameValue`].
    fn from((k, v): (Cow<'a, str>, LuaValue<'a>)) -> Self {
        if valid_lua_identifier(k.as_bytes()) {
            Self::NameValue(Box::new((k, v)))
        } else {
            Self::KeyValue(Box::new((LuaValue::String(to_utf8_cow(k)), v)))
        }
    }
}

impl<'a> TryFrom<LuaTableEntry<'a>> for (i64, LuaValue<'a>) {
    type Error = LuaTableEntry<'a>;

    /// Converts [`LuaTableEntry::KeyValue`] with [`LuaNumber::Integer`] into
    /// `(i64, LuaValue)`. Returns `Err` for other types.
    ///
    /// This is intended to help convert an `Iterator<Item = LuaTableEntry>`
    /// into a `HashMap<i64, LuaValue>`.
    fn try_from(value: LuaTableEntry<'a>) -> Result<Self, Self::Error> {
        match value {
            LuaTableEntry::KeyValue(b)
                if matches!(b.0, LuaValue::Number(LuaNumber::Integer(_))) =>
            {
                let LuaValue::Number(LuaNumber::Integer(k)) = b.0 else {
                    unreachable!();
                };

                Ok((k, b.1))
            }
            other => Err(other),
        }
    }
}

impl<'a> From<(i64, LuaValue<'a>)> for LuaTableEntry<'a> {
    /// Converts a `i64` key and [`LuaValue`] into a [`LuaTableEntry::KeyValue`]
    /// with [`LuaNumber::Integer`] key.
    fn from((k, v): (i64, LuaValue<'a>)) -> Self {
        Self::KeyValue(Box::new((k.into(), v)))
    }
}

impl<'a> TryFrom<LuaTableEntry<'a>> for (Option<i64>, LuaValue<'a>) {
    type Error = LuaTableEntry<'a>;

    /// Converts [`LuaTableEntry::KeyValue`] with [`LuaNumber::Integer`] key and
    /// [`LuaTableEntry::Value`] into `(Option<i64>, LuaValue)`.
    ///
    /// Returns `Err` for other types.
    ///
    /// This is intended to help convert an `Iterator<Item = LuaTableEntry>`
    /// into a non-consecutive array of [`LuaValue`].
    fn try_from(value: LuaTableEntry<'a>) -> Result<Self, Self::Error> {
        match value {
            LuaTableEntry::KeyValue(b)
                if matches!(b.0, LuaValue::Number(LuaNumber::Integer(_))) =>
            {
                let LuaValue::Number(LuaNumber::Integer(k)) = b.0 else {
                    unreachable!();
                };
                Ok((Some(k), b.1))
            }
            LuaTableEntry::Value(b) => Ok((None, *b)),
            other => Err(other),
        }
    }
}

impl<'a> TryFrom<LuaTableEntry<'a>> for LuaValue<'a> {
    type Error = LuaTableEntry<'a>;

    /// Converts [`LuaTableEntry::Value`] into [`LuaValue`]. Returns `Err` for other
    /// types.
    ///
    /// This is intended to help convert an `Iterator<Item = LuaTableEntry>`
    /// into an array of [`LuaValue`].
    fn try_from(value: LuaTableEntry<'a>) -> Result<Self, Self::Error> {
        match value {
            LuaTableEntry::Value(v) => Ok(*v),
            other => Err(other),
        }
    }
}

impl<'a> From<LuaValue<'a>> for LuaTableEntry<'a> {
    /// Converts [`LuaValue`] into [`LuaTableEntry::Value`].
    fn from(value: LuaValue<'a>) -> Self {
        Self::Value(Box::new(value))
    }
}
