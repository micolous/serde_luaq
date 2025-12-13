//! `serde_json` conversion routines.
use crate::{
    error::LuaConversionError,
    value::{from_utf8_cow, from_utf8_cow_lossy},
    JsonConversionError, LuaNumber, LuaTableEntry, LuaValue,
};
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};
use std::borrow::Borrow;

impl<'a> TryFrom<LuaValue<'a>> for JsonValue {
    type Error = JsonConversionError;

    fn try_from(value: LuaValue<'a>) -> Result<Self, Self::Error> {
        to_json_value(value, JsonConversionOptions::default())
    }
}

/// [Lua to JSON][to_json_value] conversion options.
#[derive(Default, Debug, PartialEq)]
pub struct JsonConversionOptions {
    /// By default, [`to_json_value()`] returns [`JsonConversionError::Utf8Error`] on invalid UTF-8
    /// sequences.
    ///
    /// When this option is set to `true`, it uses
    /// [lossy string conversion][String::from_utf8_lossy] instead. This can result in data loss.
    pub lossy_string: bool,
}

/// Converts a [`LuaValue`] into a [`serde_json::Value`].
///
/// ## Caveats
///
/// There are a number of caveats to the conversion process, which may
/// result in data loss, changes or strangely-formed data, such that running
/// `from_json_value(to_json_value(a))` _may not return the same value_.
///
/// The process aims to follow _Lua's_ conventions when coercing to JSON.
///
/// [`serde_json::Value`] always uses owned types, so conversion may result in a copy for all
/// [`Cow`][]-based types (strings).
///
/// ### Floating points
///
/// [`f64::INFINITY`], [`f64::NEG_INFINITY`] and [`f64::NAN`] cannot be represented in JSON, and
/// return [`JsonConversionError`].
///
/// ### Strings
///
/// Lua strings are
/// [assumed to be encoded as UTF-8, and converted to a `String`][std::str::from_utf8].
///
/// If the string is _not_ valid UTF-8, this will return [`JsonConversionError::Utf8Error`] (unless
/// [`JsonConversionOptions::lossy_string`] is `true`).
///
/// **Note:** Lua strings are equivalent to Rust's `[u8]` type, and have no defined encoding.
/// This can result in unexpected behaviour if the string is encoded differently or contains
/// binary data, but _could_ be interpreted as UTF-8.
///
/// JSON has no standard syntax to express arbitrary binary data, and _always_ encoding Lua
/// strings with something like Base64 would make the outputs difficult to use.
///
/// ### Tables
///
/// Lua does not have a distinct `Array`-like type, only `Object`. This method will attempt to
/// convert a `table` that looks like an `Array` into one:
///
/// * An empty table will be converted to [an empty object][JsonValue::Object].
///
/// * A table containing _only_ [implicitly-keyed entries][LuaTableEntry::Value] will be
///   converted to [an array][JsonValue::Array].
///
/// * A table containing one or more explicitly-keyed entries will be converted to
///   [an object][JsonValue::Object].
///
/// * Tables with a mix of explicitly and implicitly-keyed entries will
///   [key implicitly-keyed values in the same as Lua][0], with consecutive integers starting
///   counting at `1`, without regard for explicitly-keyed entries.
///
///   This means `{[1] = 1, 2, 3, [2] = 4}` will result in `{"1": 2, "2": 4}`
///
/// * Table keys are converted to strings when expressed as JSON:
///
///   * Lua string keys are converted in [the same way as other strings](#strings)
///   * [`nil`][LuaValue::Nil] is converted to the string `"nil"`
///   * `true` and `false` are converted to the strings `"true"` and `"false"`
///   * integers and floating points are
///     [converted to strings with _Rust_ formatting conventions][ToString::to_string]
///   * Tables keyed with a table will return [`JsonConversionError::TableKeyedWithTable`]
///
/// * Entries of tables with the same key defined multiple times will be
///   silently overwritten (later entries take precedence).
///
/// **Note:** `serde_json` may not preserve the order of keys in [an object][JsonValue::Object].
///
/// [0]: https://www.lua.org/manual/5.4/manual.html#3.4.9
/// [`Cow`]: std::borrow::Cow
pub fn to_json_value(
    value: LuaValue<'_>,
    opts: impl Borrow<JsonConversionOptions>,
) -> Result<JsonValue, JsonConversionError> {
    let opts = opts.borrow();

    match value {
        LuaValue::Nil => Ok(JsonValue::Null),

        LuaValue::String(v) => Ok(JsonValue::from(
            if opts.lossy_string {
                from_utf8_cow_lossy(v)
            } else {
                from_utf8_cow(v).map_err(|(e, _)| e)?
            }
            .to_string(),
        )),

        LuaValue::Boolean(b) => Ok(JsonValue::Bool(b)),

        LuaValue::Number(n) => JsonNumber::try_from(n).map(JsonValue::Number),

        LuaValue::Table(items) => {
            if items.is_empty() {
                // Fast-path: treat as an empty object
                return Ok(JsonValue::Object(Default::default()));
            }

            // Repeat the same process as
            // IntoLuaTableIterator::into_map_string_iter here, because that
            // way we can try to collect everything as an array first and
            // handle entry precedence correctly if we're wrong, all in a
            // single pass.
            let mut object = JsonMap::new();
            let mut array: Vec<JsonValue> = Vec::new();
            // Lua arrays start at 1
            let mut array_next_idx = 1;

            for entry in items {
                match entry {
                    LuaTableEntry::KeyValue(b) => {
                        // Switched to an object, move any existing entries from the array.
                        move_array_to_object(&mut array, &mut array_next_idx, &mut object);

                        let k = match b.0 {
                            LuaValue::String(k) => if opts.lossy_string {
                                from_utf8_cow_lossy(k)
                            } else {
                                from_utf8_cow(k).map_err(|(e, _)| e)?
                            }
                            .to_string(),
                            LuaValue::Nil => "nil".to_string(),
                            LuaValue::Boolean(k) => k.to_string(),
                            LuaValue::Number(k) => k.to_string(),
                            LuaValue::Table(_items) => {
                                return Err(JsonConversionError::TableKeyedWithTable);
                            }
                        };

                        object.insert(k, to_json_value(b.1, opts)?);
                    }

                    LuaTableEntry::NameValue(b) => {
                        // Switched to an object, move any existing entries from the array.
                        move_array_to_object(&mut array, &mut array_next_idx, &mut object);

                        object.insert(b.0.to_string(), to_json_value(b.1, opts)?);
                    }

                    LuaTableEntry::Value(v) => {
                        let v = to_json_value(*v, opts)?;
                        if object.is_empty() {
                            // We have no object yet, push into array
                            array.push(v);
                        } else {
                            // We have an object, use the next key
                            object.insert(array_next_idx.to_string(), v);
                            array_next_idx += 1;
                        }
                    }

                    LuaTableEntry::NumberValue(n) => {
                        let v = JsonNumber::try_from(n).map(JsonValue::Number)?;
                        if object.is_empty() {
                            // We have no object yet, push into array
                            array.push(v);
                        } else {
                            // We have an object, use the next key
                            object.insert(array_next_idx.to_string(), v);
                            array_next_idx += 1;
                        }
                    }

                    LuaTableEntry::BooleanValue(b) => {
                        let v = JsonValue::Bool(b);
                        if object.is_empty() {
                            // We have no object yet, push into array
                            array.push(v);
                        } else {
                            // We have an object, use the next key
                            object.insert(array_next_idx.to_string(), v);
                            array_next_idx += 1;
                        }
                    }

                    LuaTableEntry::NilValue => {
                        let v = JsonValue::Null;
                        if object.is_empty() {
                            // We have no object yet, push into array
                            array.push(v);
                        } else {
                            // We have an object, use the next key
                            object.insert(array_next_idx.to_string(), v);
                            array_next_idx += 1;
                        }
                    }
                }
            }

            match (object.is_empty(), array.is_empty()) {
                // No entries be handled by initial fast-path
                (true, true) => unreachable!(),

                // Entries in both should be handled by auto-conversion
                (false, false) => unreachable!(),

                // Entries in the object only, return the object
                (false, true) => Ok(JsonValue::Object(object)),

                // Entries in the array only, return the array
                (true, false) => Ok(JsonValue::Array(array)),
            }
        }
    }
}

/// Converts a JSON value to a Lua value.
///
/// ## Caveats
///
/// There are a number of caveats to the conversion process, which may
/// result in data loss, changes or strangely-formed data, such that running
/// `to_json_value(from_json_value(a))` _may not return the same value_.
///
/// ### Numbers
///
/// `serde_json` stores integers using either `i64` or `u64`. Integers `> i64::MAX` will be
/// casted to `f64`, resulting in a loss of precision, but JSON's `Number` type is an `f64` anyway.
///
/// `serde_json` drops non-finite numbers. When its `arbitrary_precision` feature is enabled, it
/// will store non-finite numbers, but will still drop them when calling
/// [`as_f64`][JsonValue::as_f64].
///
/// ### Arrays
///
/// [Arrays][JsonValue::Array] are converted to a [`LuaValue::Table`] with [`LuaTableEntry::Value`]
/// entries (ie: implicit keys).
///
/// ### Objects
///
/// [Objects][JsonValue::Object] are converted a [`LuaValue::Table`] in the order `serde_json`
/// returned (which may not be the same as the order in the file).
///
/// Entries are a [`LuaTableEntry::NameValue`] if the object's key is a valid Lua identifier,
/// or [`LuaTableEntry::KeyValue`] otherwise.
pub fn from_json_value(value: JsonValue) -> Result<LuaValue<'static>, LuaConversionError> {
    match value {
        JsonValue::Null => Ok(LuaValue::Nil),
        JsonValue::Bool(b) => Ok(LuaValue::Boolean(b)),
        JsonValue::Number(n) => {
            if let Some(v) = n.as_i64() {
                Ok(LuaValue::integer(v))
            } else if let Some(v) = n.as_f64() {
                Ok(LuaValue::float(v))
            } else {
                Err(LuaConversionError::Number)
            }
        }
        JsonValue::String(s) => Ok(LuaValue::String(s.into_bytes().into())),
        JsonValue::Array(a) => {
            let r: Result<Vec<LuaTableEntry<'static>>, LuaConversionError> = a
                .into_iter()
                .map(|e| Ok(from_json_value(e)?.into()))
                .collect();

            Ok(r?.into())
        }
        JsonValue::Object(o) => {
            let r: Result<Vec<LuaTableEntry<'static>>, LuaConversionError> = o
                .into_iter()
                .map(|(k, v)| Ok(LuaTableEntry::from((k, from_json_value(v)?))))
                .collect();

            Ok(r?.into())
        }
    }
}

#[inline]
fn move_array_to_object(
    array: &mut Vec<JsonValue>,
    array_next_idx: &mut i64,
    object: &mut JsonMap<String, JsonValue>,
) {
    if !array.is_empty() {
        let array = std::mem::take(array);
        for v in array {
            object.insert(array_next_idx.to_string(), v);
            *array_next_idx += 1;
        }
    }
}

impl TryFrom<LuaNumber> for JsonNumber {
    type Error = JsonConversionError;

    /// Convert a [`LuaNumber`] to JSON Number.
    ///
    /// [`f64::INFINITY`], [`f64::NEG_INFINITY`] and [`f64::NAN`] cannot be represented in JSON, and
    /// return [`JsonConversionError`].
    fn try_from(value: LuaNumber) -> Result<Self, Self::Error> {
        match value {
            LuaNumber::Integer(i) => Ok(JsonNumber::from(i)),

            LuaNumber::Float(f) => match JsonNumber::from_f64(f) {
                Some(o) => Ok(o),
                None => {
                    if f.is_infinite() {
                        if f.is_sign_positive() {
                            Err(JsonConversionError::PositiveInfinity)
                        } else {
                            Err(JsonConversionError::NegativeInfinity)
                        }
                    } else if f.is_nan() {
                        Err(JsonConversionError::NaN)
                    } else {
                        // shouldn't happen!
                        Err(JsonConversionError::Float)
                    }
                }
            },
        }
    }
}
