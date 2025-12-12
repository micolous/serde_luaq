//! `serde_luaq` is a library for deserialising (and eventually, serialising) simple, JSON-like data
//! structures from Lua 5.4 source code, _without requiring Lua itself_.
//!
//! The goal is to be able to read state from software (mostly games) which is serialised using
//! [Lua `%q` formatting][format] (and similar techniques) _without_ requiring arbitrary code
//! execution.
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
//! - A [Serde]-based `Deserialize` implementation for
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
//! assert_eq!(LuaValue::Boolean(true), lua_value(b"true", /* max table depth */ 16).unwrap());
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
//!
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
//!         b"{a=true, [ [[b]] ]={[3] = 3, 0x1, 2}, ['c'] = { foo = \"bar\" }}",
//!         LuaFormat::Value,
//!         /* maximum table depth */ 16,
//!     ).unwrap(),
//! );
//! ```
//!
//! ## Data types
//!
//! `serde_luaq` supports a JSON-like subset of Lua 5.4's data types:
//!
//! | Lua type  | [`LuaValue`] variant    | Rust type(s) for Serde             |
//! | --------- | ----------------------- | ---------------------------------- |
//! | `nil`     | [`LuaValue::Nil`][]     | [`Option::None`][]                 |
//! | `boolean` | [`LuaValue::Boolean`][] | [`bool`][]                         |
//! | `string`  | [`LuaValue::String`][]  | `[u8]`, `Vec<u8>`, [`String`] ([see note](#strings)) |
//! | `number`  | [`LuaValue::Number`][]  | [`LuaNumber`][] ([see note](#numbers))               |
//! | ...`float` subtype   | [`LuaNumber::Float`][]   | [`f64`][]              |
//! | ...`integer` subtype | [`LuaNumber::Integer`][] | [`i64`][]              |
//! | `table`   | [`LuaValue::Table`][]   | [`BTreeMap`][std::collections::BTreeMap], [`HashMap`][std::collections::HashMap], [`Vec<T>`] ([see note](#tables))|
//!
//! The [`peg` deserialisers](#peg-deserialiser) will always produce a [`LuaValue`][].
//!
//! However, [`LuaValue` doesn't implement `Deserialize`][LuaValue#serde], so can't be used as a
//! Serde field.
//!
//! ### Numbers
//!
//! `serde_luaq` follows Lua 5.4's number handling semantics. The following types can be used with
//! Serde's data model:
//!
//! | Literal | [`LuaNumber`][] | [`f64`][] | [`i64`][] |
//! | ------- | :-------------: | :-------: | :-------: |
//! | Decimal integer,<br>inside [`i64`][] range | ✅ | may lose precision | ✅ |
//! | Decimal integer,<br>outside [`i64`][] range | will lose precision | will lose precision | ❌ |
//! | Hexadecimal integer | ✅ | may lose precision | ✅ |
//! | Decimal float | ✅ | ✅ | ❌ |
//! | Hexadecimal float[^wasm] | ✅ | ✅ | ❌ |
//! | `(0/0)` (NaN) | ✅ | ✅ | ❌ |
//!
//! [^wasm]: Not supported on WASM targets
//!
//! * A [`LuaNumber`][] field will follow Lua 5.4 semantics, which could be a [`i64`][] or
//!   [`f64`][].
//!
//! * An [`f64`][] field will accept decimal integer literals from &minus;(2<sup>53</sup> &minus; 1)
//!   to (2<sup>53</sup> &minus; 1) without loss of precision.
//!
//! * Decimal integer literals _outside_ of the [`i64`][] range are converted to [`f64`][], and will
//!   lose precision. These cannot be used with [`i64`][] fields.
//!
//! * Hexadecimal integer literals _outside_ of the [`i64`][] range will under/overflow as
//!   [`i64`][], even in [`f64`][] fields (ie: `0xffffffffffffffff` is `-1_f64`), and can _always_
//!   be used with [`i64`][] fields.
//!
//! * Unsigned integer fields like [`u8`][] and [`u16`][] reject all negative integer literals,
//!   including hexadecimal integer literals.
//!
//! * Narrower integer fields like [`i8`][] and [`i16`][] reject all integer literals outside of
//!   their range, including hexadecimal integer literals.
//!
//! * Narrower float fields like [`f32`][] are first handled as a [`f64`][] then converted to
//!   [`f32`][]. This will result in a loss of precision, and values outside of
//!   [their acceptable range][f32::MAX] will be set to [positive][f32::INFINITY] or
//!   [negative infinity][f32::NEG_INFINITY].
//!
//! * Wider integer fields like [`i128`][] and [`u64`][] apply the same limits as [`i64`][], even
//!   with hexadecimal integer literals.
//!
//! ### Strings
//!
//! Lua strings are "8-bit clean", and can contain *any* 8-bit value (ie: `[u8]`).
//!
//! For Serde, this is preserved if using a `Vec<u8>` field
//! [with `#[serde(with = "serde_bytes")]` or a `serde_bytes::ByteBuf` field][serde_bytes]. If you
//! _don't_ use [`serde_bytes`][serde_bytes], Serde will expect a sequence of [`u8`][] (and won't
//! read the string).
//!
//! Lua's `\u{...}` escapes follow [RFC 2279][] (1998) rather than [RFC 3629][] (2003). RFC 2279
//! differs by allowing [surrogate code points][surrogate] and code points greater than
//! `\u{10FFFF}`. `serde_luaq` will convert these escapes into bytes following RFC 2279.
//!
//! Serde [`String`] fields can be used the string literal evaluates to valid RFC 3629 UTF-8. This
//! is not guaranteed even if [the input data is `&str`][self::from_str], as Lua string escapes may
//! evaluate to binary values or invalid sequences (eg: `"\xC1\u{7FFFFFFF}"`).
//!
//! ### Tables
//!
//! Lua tables are used for both lists and maps.
//!
//! The [`peg` deserialisers](#peg-deserialiser) will always produce a [`Vec`][] of
//! [`LuaTableEntry`][] in the order the entries were defined, including duplicate keys.
//!
//! #### Duplicate table keys in Serde
//!
//! Using duplicate table keys is undefined behaviour in Lua.
//!
//! However, when using `serde_luaq` with Serde, later entries always overwrite earlier entries,
//! regardless of how they are defined, ie:
//!
//! ```lua
//! { ['a'] = 1, a = 2 } == { ['a'] = 2 }
//! { 1, [1] = 2 } == { 2 }
//! { [1] = 1, 2 } == { 2 }
//! ```
//!
//! #### Tables as lists in Serde (Vec)
//!
//! Lua tables are 1-indexed, rather than 0-indexed. `serde_luaq` will handle these differences on
//! the input side for explicitly-keyed values, and make the resulting [`Vec`] 0-indexed.
//!
//! Table entries may be defined with implicit or explicit keys, or a combination. Any missing
//! entries will be treated as `nil`:
//!
//! ```text
//! { 1, [3] = 2 } == vec![Some(1), None, Some(2)]
//! ```
//!
//! #### Tables as maps in Serde (BTreeMap/HashMap)
//!
//! If the key of the map is an integer type, table entries may contain implicit keys. Like Lua,
//! implicit keys start counting at 1, without regard for explicit keys.
//!
//! Otherwise, all entries must be explicitly keyed.
//!
//! #### Tables as structs
//!
//! When deserialising a table as a `struct`, all keys must be strings or
//! [Lua identifiers][LuaTableEntry::NameValue].
//!
//! [Serde does not support numeric keys in structs][serde-num-keys].
//!
//! ### Using with older / other versions of Lua
//!
//! As `serde_luaq` does not execute Lua code, there are only a small number of compatibility
//! issues:
//!
//! * **Lua 5.3** over/underflows decimal integers that don't fit in a [`i64`][], rather than
//!   coercing to [`f64`][].
//!
//!   Hexadecimal integers over/underflow in both Lua 5.3 and 5.4.
//!
//! * **Lua 5.2 and earlier, and Luau** always use [`f64`][] for numbers, and do not have an integer
//!   subtype.
//!
//! * **Lua 5.1 and earlier** allow locale-dependent letters in identifiers (rather than just
//!   basic Latin), and `goto` is not a reserved keyword.
//!
//!   This affects [table entries in the form `{foo = bar}`][LuaTableEntry::NameValue]
//!   and parsing in script mode.
//!
//! * **Luau** also adds type annotations, binary integer literals, separators for all integer
//!   literals and string interpolation. None of these features are supported by `serde_luaq`.
//!
//! ## Maximum table depth
//!
//! The maximum table depth argument (`max_depth`) controls how deeply nested a table can be before
//! being rejected by `serde_luaq`. Set this to the maximum depth of tables that you expect in your
//! input data.
//!
//! For example:
//!
//! ```lua
//! -- Table of depth 1
//! a1 = {1, 2, 3}
//!
//! -- An empty table is still of depth 1
//! b1 = {}
//!
//! -- Table of depth 2
//! a2 = {
//!     {1, 2, 3},
//!     {4, 5, 6},
//! }
//! ```
//!
//! **Warning:** setting this value too high allows a heavily-nested table to cause your program
//! [to overflow its stack and crash][stackoverflow]. What is "too high" depends on your platform
//! and where you call `serde_luaq` in your program.
//!
//! Setting `max_depth` to `0` disables support for tables, _even empty tables_.
//!
//! This is roughly equivalent to Lua's `LUAI_MAXCCALLS` build option, which counts many other
//! nested lexical elements which `serde_luaq` doesn't support (like code blocks and parentheses).
//!
//! [format]: https://www.lua.org/manual/5.4/manual.html#pdf-string.format
//! [`peg`]: https://docs.rs/peg/latest/peg/
//! [Serde]: https://serde.rs/
//! [serde_bytes]: https://docs.rs/serde_bytes/latest/serde_bytes/
//! [serde-num-keys]: https://github.com/serde-rs/serde/issues/2358
//! [surrogate]: https://www.unicode.org/versions/Unicode17.0.0/core-spec/chapter-3/#G2630
//! [RFC 2279]: https://www.rfc-editor.org/rfc/rfc2279
//! [RFC 3629]: https://www.rfc-editor.org/rfc/rfc3629
//! [stackoverflow]: https://github.com/rust-lang/rust/issues/79935
mod de;
mod error;
mod number;
mod peg_parser;
#[cfg(feature = "serde_json")]
mod serde_json;
mod table_entry;
mod value;

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

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
/// Converts a string to a `f64` using C's standard library.
///
/// This supports parsing hexadecimal floating points.
fn strtod(i: &str) -> Option<f64> {
    use std::ffi::{c_char, CString};

    extern "C" {
        fn strtod(nptr: *const c_char, endptr: &mut usize) -> f64;
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
