//! `serde_luaq` is a library for deserialising (and eventually, serialising) simple, JSON-like data
//! structures from Lua source code, _without requiring Lua itself_.
//!
//! The goal is to be able to read state from software (mostly games) which is serialised using
//! [Lua `%q` formatting][format].
//!
//! This library consists of four parts:
//!
//! - A [`LuaValue`] `enum`, which describes Lua's basic data types ([`nil`][LuaValue::Nil],
//!   [boolean][LuaValue::Boolean], [string][LuaValue::String], [number][LuaValue::Number],
//!   [table][LuaValue::Table]).
//!
//! - A [`peg`]-based parser for parsing a `&[u8]` (containing Lua) into a `LuaValue` from
//!   [a bare Lua value expression][lua_value], [a single `return` statement][return_statement] or
//!   [script with variable assignments][script].
//!
//! - A [`serde`]-based `Deserialize` implementation for
//!   [converting a `LuaValue`][from_slice] into your own data types.
//!
//! - _Optional_ lossy [converter to][to_json_value] and [from][from_json_value] `serde_json`'s
//!   `Value` type.
//!
//! ## Examples
//!
//! ### peg deserialiser
//!
//! Deserialise [a bare Lua value][lua_value] with the [`peg`] parser to [`LuaValue`]:
//!
//! ```rust
//! use serde_luaq::{LuaValue, lua_value};
//! assert_eq!(LuaValue::Boolean(true), lua_value(b"true").unwrap());
//! ```
//!
//! There are similar deserialisers for [a single `return` statement][return_statement] and
//! [scripts with one or more variable assignments][script].
//!
//! ### serde deserialiser
//!
//! ```rust
//! use serde::Deserialize;
//! use serde_luaq::{LuaFormat, from_slice};
//! #[derive(Deserialize, PartialEq, Debug)]
//! struct ComplexType {
//!     foo: String,
//! }
//!
//! #[derive(Deserialize, PartialEq, Debug)]
//! struct Test {
//!     a: bool,
//!     b: Vec<u32>,
//!     c: ComplexType,
//! }
//!
//! let expected = Test {
//!     a: true,
//!     b: vec![1, 2, 3],
//!     c: ComplexType { foo: "bar".to_string() },
//! };
//!
//! assert_eq!(
//!     expected,
//!     from_slice(
//!         b"{a=true, [[[b]]]={[3] = 3, 0x1, 2}, ['c'] = { foo = \"bar\" }}",
//!         LuaFormat::Value,
//!     ).unwrap(),
//! );
//! ```
//!
//! [`peg`]: https://docs.rs/peg/latest/peg/
//! [`serde`]: https://serde.rs/
//! [format]: https://www.lua.org/manual/5.4/manual.html#pdf-string.format
mod de;
mod error;
mod number;
mod peg_parser;
#[cfg(feature = "serde_json")]
mod serde_json;
mod table_entry;
mod value;

use std::ffi::CString;

pub use crate::{
    de::{from_slice, from_str, LuaFormat},
    error::{Error, Result},
    number::LuaNumber,
    peg_parser::lua::{lua_value, return_statement, script},
    table_entry::LuaTableEntry,
    value::LuaValue,
};

#[cfg(feature = "serde_json")]
pub use crate::{
    error::{JsonConversionError, LuaConversionError},
    serde_json::{from_json_value, to_json_value, JsonConversionOptions},
};

/// Sorted list of Lua keywords which cannot be used as field names in scripts.
///
/// Reference: <https://www.lua.org/manual/5.4/manual.html#3.1>
const LUA_KEYWORDS: [&[u8]; 22] = [
    b"and",
    b"break",
    b"do",
    b"else",
    b"elseif",
    b"end",
    b"false",
    b"for",
    b"function",
    b"goto",
    b"if",
    b"in",
    b"local",
    b"nil",
    b"not",
    b"or",
    b"repeat",
    b"return",
    b"then",
    b"true",
    b"until",
    b"while",
];

/// Returns `true` if `i` is a valid Lua identifier.
///
/// Per [Lua's _Lexical Conventions_][0]:
///
/// > _Names_ (also called _identifiers_) in Lua can be any string of Latin letters, Arabic-Indic
/// > digits, and underscores, not beginning with a digit and not being a reserved word.
/// >
/// > Identifiers are used to name variables, table fields, and labels.
///
/// While Lua allows non-UTF-8-encoded data, a valid Lua identifier _is_ valid UTF-8.
///
/// [0]: https://www.lua.org/manual/5.4/manual.html#3.1
fn valid_lua_identifier(i: &[u8]) -> bool {
    if i.is_empty() || LUA_KEYWORDS.binary_search(&i).is_ok() {
        return false;
    }

    let mut i = i.iter();
    let Some(&first_char) = i.next() else {
        return false;
    };

    if !(first_char.is_ascii_alphabetic() || first_char == b'_') {
        return false;
    }

    i.all(|&c| c.is_ascii_alphanumeric() || c == b'_')
}

/// Converts a string to a `f64` using C's standard library.
///
/// This supports parsing hexadecimal floating points.
fn strtod(i: &str) -> Option<f64> {
    extern "C" {
        fn strtod(nptr: *const i8, endptr: &mut usize) -> f64;
    }

    // Length excludes null byte
    let len = i.len();

    // Copy to local buffer with null terminator
    let i = CString::new(i).ok()?;
    let nptr = i.as_ptr();
    let expected_endptr = nptr.addr() + len;

    // strtod does not use this as an input
    let mut endptr = 0;

    let o = unsafe { strtod(nptr, &mut endptr) };

    if expected_endptr != endptr {
        // strtod didn't parse the whole value, so there was some error
        None
    } else {
        Some(o)
    }
}

/// Parses a `&[u8]` as a byte-string containing an integer expressed using
/// ASCII `0-9`, `A-Z` and `a-z`, wrapping on overflow or underflow (like Lua).
///
/// This is an adaptation of Rust's [`from_str_radix`][0], though probably much
/// less optimised. :)
///
/// Returns `None` if `src` contains invalid characters.
///
/// Panics if `radix` is not in the range 2 to 36.
///
/// [0]: i64::from_str_radix
fn wrapping_parse_int(digits: &[u8], radix: u32, is_positive: bool) -> Option<i64> {
    if !(2..=36).contains(&radix) {
        panic!("invalid radix: {radix}");
    }

    let mut result = 0i64;
    for &c in digits {
        let x = (c as char).to_digit(radix)? as i64;
        result = result.wrapping_mul(radix as i64);
        if is_positive {
            result = result.wrapping_add(x);
        } else {
            result = result.wrapping_sub(x);
        }
    }

    Some(result)
}

#[cfg(test)]
mod test {
    use crate::LUA_KEYWORDS;

    /// Ensure the list of Lua keywords is sorted. This allows us to use
    /// [`binary_search()`][0] to match keywords, rather than [`contains()`][1].
    ///
    /// [0]: https://doc.rust-lang.org/std/primitive.slice.html#method.binary_search
    /// [1]: https://doc.rust-lang.org/std/primitive.slice.html#method.contains
    #[test]
    fn sorted_keywords() {
        assert!(LUA_KEYWORDS.is_sorted());
    }
}
