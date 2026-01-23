use serde::{de, ser};
use std::fmt::Display;
#[cfg(feature = "serde_json")]
use std::str::Utf8Error;
use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, ThisError, PartialEq, Eq)]
pub enum Error {
    #[error("serde deserialize error: {0}")]
    SerdeDeserialize(String),
    #[error("serde serialize error: {0}")]
    SerdeSerialize(String),
    #[error("peg parse error: {0:?}")]
    Peg(#[from] peg::error::ParseError<usize>),
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::SerdeSerialize(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::SerdeDeserialize(msg.to_string())
    }
}

#[cfg(feature = "serde_json")]
/// Errors when converting Lua to JSON.
#[derive(Debug, ThisError, PartialEq)]
pub enum JsonConversionError {
    #[error("positive infinity cannot be represented in standard JSON")]
    PositiveInfinity,

    #[error("negative infinity cannot be represented in standard JSON")]
    NegativeInfinity,

    #[error("NaN cannot be represented in standard JSON")]
    NaN,

    #[error("unknown floating point conversion failure")]
    Float,

    /// Lua value contained a string that could not be encoded as RFC 3629 UTF-8.
    /// 
    /// Set [`JsonConversionOptions::lossy_string`][crate::JsonConversionOptions::lossy_string]
    /// to use [lossy string conversion][String::from_utf8_lossy] instead. This can result in data
    /// loss.
    #[error("UTF-8 encoding error: {0:?}")]
    Utf8Error(#[from] Utf8Error),

    #[error("Lua table contains a table as a key")]
    TableKeyedWithTable,
    
    /// Lua table was keyed with `nil`.
    /// 
    /// Set
    /// [`JsonConversionOptions::nil_key_to_string`][crate::JsonConversionOptions::nil_key_to_string]
    /// to convert it to the string `"nil"`.
    #[error("Lua table contains nil as a key")]
    TableKeyedWithNil,

    /// Lua table was keyed with `true` or `false`.
    /// 
    /// Set
    /// [`JsonConversionOptions::boolean_key_to_string`][crate::JsonConversionOptions::boolean_key_to_string]
    /// to convert it to the string `"true"` or `"false"`.
    #[error("Lua table contains boolean as a key")]
    TableKeyedWithBoolean,

    /// Lua table was keyed with a float.
    /// 
    /// Set
    /// [`JsonConversionOptions::integer_key_to_string`][crate::JsonConversionOptions::integer_key_to_string]
    /// to convert it to a string in decimal form.
    #[error("Lua table contains integer as a key")]
    TableKeyedWithInteger,

    #[error("Lua table contains float as a key")]
    TableKeyedWithFloat,

    #[error("Lua table contains mix of implicit and explicit keys")]
    TableKeyedImplicitlyAndExplicitly,
}

#[cfg(feature = "serde_json")]
/// Errors when converting JSON to Lua.
#[derive(Debug, ThisError, PartialEq)]
pub enum LuaConversionError {
    #[error("Lua numbers must fit in `i64` or `f64`")]
    Number,
}
