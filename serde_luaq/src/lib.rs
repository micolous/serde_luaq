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
//! Generally speaking, `serde_luaq` tries to do whatever a default build of Lua 5.4 does,
//! **except for**:
//!
//! * anything which requires evaluating or executing Lua code
//! * locale-dependant behaviour
//! * platform-dependant behaviour
//!
//! Unicode identifiers (`LUA_UCID`) and other locale-specific identifiers are not supported, even
//! if they would be valid in Rust.
//!
//! ### Numbers
//!
//! `serde_luaq` follows Lua 5.4's number handling semantics, but _doesn't_ implement
//! locale-specific behaviour (eg: [using `,` as a decimal point in addition to `.`][comma]).
//!
//! The following types can be used with Serde's data model:
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
//! * Hexadecimal integer literals are always coerced to [`i64`][], and can _always_ be used with
//!   [`i64`][] fields. Values _outside_ of the [`i64`][] range will _only_ under/overflow as
//!   [`i64`][], regardless of the field type.
//!
//!   This means the literal `0xffffffffffffffff` is always treated as if it were written `-1`, even
//!   for [`f64`][], [`i8`][], and [`u64`][] fields. This would be an error for unsigned types.
//!
//! * Narrower integer fields like [`i8`][] and [`i16`][] reject all integer literals that are
//!   outside of their range.
//!
//!   The [`i64`][] coersion process means that the hexadecimal literal `0xff` is treated as if it
//!   were written `255`, and using it with an [`i8`][] field would be an error.
//!
//! * Unsigned integer fields like [`u8`][] and [`u16`][] reject all negative decimal integer
//!   literals.
//!
//! * Narrower float fields like [`f32`][] are first handled as a [`f64`][], then converted to
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
//! `\u{10FFFF}`. `serde_luaq` will convert these escapes into bytes following RFC 2279, which might
//! not be valid in RFC 3629.
//!
//! Serde [`String`] fields can be used the string literal evaluates to valid RFC 3629 UTF-8. This
//! is not guaranteed even if [the input data is `&str`][self::from_str], as Lua string escapes may
//! evaluate to binary values or invalid sequences (eg: `"\xC1\u{7FFFFFFF}"`).
//!
//! **Unlike Lua,** new-line characters/sequences in strings are kept _as-is_, and not converted to
//! their platform-specific representation.
//!
//! ### Tables
//!
//! Lua tables are used for both lists and maps.
//!
//! The [`peg` deserialisers](#peg-deserialiser) will always produce a [`Vec`][] of
//! [`LuaTableEntry`][] in the order the entries were defined. It does not attempt to reconcile
//! implicit keys mixed with explicit keys, nor duplicate keys.
//!
//! Unlike Lua, a [`LuaTableEntry`][] may use _any_ key or value type, including
//! [`nil`][LuaValue::Nil] and [NaN][f64::NAN].
//!
//! As a convienience, [identifier-keyed entries][LuaTableEntry::NameValue] (`{ a = 1 }`) are
//! treated as keyed with [`str`][], because with Lua's default build settings, these are always
//! valid [RFC 3629][] UTF-8.
//!
//! The rules are slightly different when using Serde, which is described below.
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
//! the input side for explicitly-keyed values, and make the resulting [`Vec`] 0-indexed:
//!
//! ```rust
//! # use serde::Deserialize;
//! # use serde_luaq::{Error, LuaFormat, from_slice};
//! # fn main() -> Result<(), Error> {
//! let a: Vec<i64> = from_slice(
//!     b"{1, 2, 3}",
//!     LuaFormat::Value,
//!     /* max table depth */ 16,
//! )?;
//!
//! assert_eq!(a[0], 1);
//! assert_eq!(a[1], 2);
//! assert_eq!(a[2], 3);
//! # Ok(())
//! # }
//! ```
//!
//! Table entries may be defined with implicit or explicit keys, or a combination, and may be
//! defined in any order.
//!
//! Any missing entries will be treated as `nil`, which can be used with [`Option`][]:
//!
//! ```rust
//! # use serde::Deserialize;
//! # use serde_luaq::{Error, LuaFormat, from_slice};
//! # fn main() -> Result<(), Error> {
//! let b: Vec<Option<i64>> = from_slice(
//!     b"{1, [4] = 4, 2}",
//!     LuaFormat::Value,
//!     /* max table depth */ 16,
//! )?;
//!
//! assert_eq!(b, vec![Some(1), Some(2), None, Some(4)]);
//!
//! let c: Vec<Option<i64>> = from_slice(
//!     b"{1, [1000] = 1000}",
//!     LuaFormat::Value,
//!     /* max table depth */ 16,
//! )?;
//!
//! assert_eq!(c.len(), 1000);
//! assert_eq!(c[0], Some(1));
//! assert_eq!(c[999], Some(1000));
//! # Ok(())
//! # }
//! ```
//!
//! If you're working with a sparse table, it's probably better to handle it as a map (see below).
//!
//! #### Tables as maps in Serde (BTreeMap/HashMap)
//!
//! If the key of the map is an integer type, table entries may contain implicit keys. Like Lua,
//! implicit keys start counting at 1, without regard for explicit keys.
//!
//! ```rust
//! # use std::collections::BTreeMap;
//! # use serde::Deserialize;
//! # use serde_luaq::{Error, LuaFormat, from_slice};
//! # fn main() -> Result<(), Error> {
//! let a: BTreeMap<i64, i64> = from_slice(
//!     b"{1, [4] = 4, 2}",
//!     LuaFormat::Value,
//!     /* max table depth */ 16,
//! )?;
//!
//! assert_eq!(1, *a.get(&1).unwrap());
//! assert_eq!(2, *a.get(&2).unwrap());
//! assert_eq!(4, *a.get(&4).unwrap());
//! # Ok(())
//! # }
//! ```
//!
//! Otherwise, all entries must be explicitly keyed.
//!
//! For maps, `serde_luaq` treats "entry present and set to `nil`" and "entry not present" as
//! distinct states. This means unless a key or value uses an [`Option`][] type, it must not contain
//! `nil`:
//!
//! ```rust
//! # use std::collections::BTreeMap;
//! # use serde::Deserialize;
//! # use serde_luaq::{Error, LuaFormat, from_slice};
//! # fn main() -> Result<(), Error> {
//! let input = b"{a = 1, b = nil}";
//!
//! // Error: b cannot be a unit (None) type
//! assert!(from_slice::<BTreeMap<String, i64>>(input, LuaFormat::Value, 16).is_err());
//!
//! // Success: b is set to None, other entries are set to Some
//! let a: BTreeMap<String, Option<i64>> = from_slice(input, LuaFormat::Value, 16)?;
//! assert_eq!(Some(1), *a.get("a").unwrap());
//! assert!(a.get("b").unwrap().is_none()); // present, set to nil
//! assert!(a.get("c").is_none()); // not present
//! # Ok(())
//! # }
//! ```
//!
//! #### Tables as structs
//!
//! When deserialising a table as a `struct`, all keys must be valid [RFC 3629 strings](#strings) or
//! [Lua identifiers][LuaTableEntry::NameValue].
//!
//! Unicode identifiers (`LUA_UCID`) and other locale-specific identifiers are not supported, even
//! if they would be valid Rust identifiers. If used in a table key, these must be written as a
//! string instead:
//!
//! ```lua
//! { english = "en", ["français"] = "fr" }
//! ```
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
//! * **Ravi** adds type annotations and some other language features, which aren't supported by
//!   `serde_luaq`.
//!
//! ## Security
//!
//! While using Lua as a serialisation format is convenient to work with in Lua,
//! [its `load()`][load] and [`require()`][require] functions allow arbitrary code execution, so
//! aren't safe to use with untrusted inputs. These risks are similar to using
//! [JavaScript's `eval()` function][jseval] to load JSON data (instead of
//! [`JSON.parse()`][jsonparse]).
//!
//! For example, this Lua function loads an expression in the string `data`, similar to what would
//! be produced by [the `serialize()` function described in _Programming in Lua_][pil12.1.1]:
//!
//! ```lua
//! -- WARNING: this function is insecure and unsafe.
//! function deserialize(data)
//!     data = "return (" .. data .. ")"
//!     local f = load(data, nil, "t")
//!     if f == nil then
//!         return error("could not load data")
//!     end
//!     local status, r = pcall(f)
//!     if not status then
//!         return error("could not call data")
//!     end
//!     return r
//! end
//!
//! a = deserialize("{hello='world'}")
//! -- Prints "world"
//! print(a.hello)
//! ```
//!
//! If your program is ever sent untrusted Lua inputs, a malicious actor could insert some code
//! which could do anything to your program or the system it is running on.
//!
//! For example, this input would cause it to read and return the contents of `/etc/passwd`:
//!
//! ```lua
//! (function() f=io.open('/etc/passwd');return f:read('a');end)()
//! ```
//!
//! `serde_luaq` addresses this risk by implementing a JSON-like subset of Lua's syntax, such that
//! inserting code is a syntax error:
//!
//! ```rust
//! use serde_luaq::{LuaValue, lua_value};
//!
//! // This would cause Lua to return the contents of a local file:
//! let input = b"(function() f=io.open('/etc/passwd');return f:read('a');end)()";
//! // But it's a syntax error here.
//! assert!(lua_value(input, 16).is_err());
//!
//! // This would cause Lua to use a lot of RAM:
//! let input = b"(function() x={};for a=1,100000000 do x[a]=a end;return x;end)()";
//! // But it's a syntax error here.
//! assert!(lua_value(input, 16).is_err());
//! ```
//!
//! Ideally, [`serde_luaq` shouldn't use significantly more memory than Lua](#large-input-data) to
//! read the same data structures. If it doesn't, that's a bug. :)
//!
//! ### Maximum table depth
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
//! This is roughly equivalent to Lua's `LUAI_MAXCCALLS` build option, which counts many other
//! nested lexical elements which `serde_luaq` doesn't support (like code blocks and parentheses).
//!
//! <div class="warning">
//!
//! **Warning:** setting `max_depth` too high allows a heavily-nested table to cause your program
//! [to overflow its stack and crash][stackoverflow], or use
//! [a large amount of memory per byte of input Lua](#large-input-data).
//!
//! What is "too high" depends on your platform and where you call `serde_luaq` in your program.
//!
//! Setting `max_depth` to `0` disables support for tables, _even empty tables_.
//!
//! </div>
//!
//! ### Large input data
//!
//! `serde_luaq` requires that the entire input fit in memory, and be less than [`usize::MAX`][]
//! bytes (4 GiB on 32-bit systems, 16 EiB on 64-bit systems). It is the _caller's_ responsibility
//! to enforce a reasonable input size limit.
//!
//! When deserialising a Lua data structure on a 64-bit system, the _minimum_ sizes of the
//! [`LuaValue`][] and [`LuaTableEntry`][] `enum`s are 32 and 16 bytes respectively. Values are
//! checked at compile-time on `aarch64`, `wasm32` and `x86_64` targets to prevent regressions.
//!
//! Heap-allocated variants of these `enum`s (those with [`Cow`][std::borrow::Cow] or [`Vec`][]
//! fields) will use more memory.
//!
//! `serde_luaq` uses [`Cow`][std::borrow::Cow] to avoid owning strings whenever possible, and
//! borrow them from the input buffer instead. However, strings that contain escape sequences need
//! to be copied, but this is _at worst_ 1 to 1 memory with input Lua plus [`Vec`][]'s usual
//! overheads.
//!
//! At present, the highest-known memory usage per byte of input Lua is a deeply-nested table of
//! tables, which consumes about 96 bytes of RAM for 2 bytes of input Lua (ie: 48&times;) on a
//! 64-bit system. This means a 64 MiB input could use up to 3 GiB of RAM.
//!
//! Lua uses similar amounts of memory for such data structures.
//!
//! [comma]: https://github.com/lua/lua/blob/104b0fc7008b1f6b7d818985fbbad05cd37ee654/testes/literals.lua#L298-L300
//! [format]: https://www.lua.org/manual/5.4/manual.html#pdf-string.format
//! [`peg`]: https://docs.rs/peg/latest/peg/
//! [jseval]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/eval
//! [jsonparse]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON/parse
//! [load]: https://www.lua.org/manual/5.4/manual.html#pdf-load
//! [pil12.1.1]: https://www.lua.org/pil/12.1.1.html
//! [require]: https://www.lua.org/manual/5.4/manual.html#pdf-require
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
