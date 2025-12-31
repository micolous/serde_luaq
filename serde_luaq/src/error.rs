use serde::{de, ser};
use std::fmt::Display;
#[cfg(feature = "serde_json")]
use std::str::Utf8Error;
use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("serde deserialize error: {0}")]
    SerdeDeserialize(String),
    #[error("serde serialize error: {0}")]
    SerdeSerialize(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("peg parse error: {0:?}")]
    Peg(#[from] peg::error::ParseError<usize>),
    #[error("invalid Lua identifier: {0:?}")]
    InvalidLuaIdentifier(String),
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

    #[error("UTF-8 encoding error: {0:?}")]
    Utf8Error(#[from] Utf8Error),

    #[error("Lua table contains a table as a key")]
    TableKeyedWithTable,
}

#[cfg(feature = "serde_json")]
/// Errors when converting JSON to Lua.
#[derive(Debug, ThisError, PartialEq)]
pub enum LuaConversionError {
    #[error("Lua numbers must fit in `i64` or `f64`")]
    Number,
}
