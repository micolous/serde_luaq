//! > **Note:** this library is still a work in progress, and there are no API stability guarantees.
//!
//! `serde_luaq` is a library for deserialising (and eventually, serialising) simple, JSON-like data
//! structures from Lua 5.4 source code, _without requiring Lua itself_.
//!
//! The goal is to safely read state from software (mostly games) which is serialised using
//! [Lua `%q` formatting][format] (and similar techniques)
//! [_without_ allowing arbitrary code execution](#security).
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
//! There are similar deserialisers for [a `return` statement][return_statement] and
//! [scripts with one or more variable assignments][script].
//!
//! [Maximum table depth limits are described in their own section](#maximum-table-depth).
//!
//! ### serde deserialiser
//!
//! [`from_slice()`][] deserialises a [a bare Lua value][LuaFormat::Value] into a type that
//! implements [`Deserialize`][serde::Deserialize]:
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
//! It can also deserialise from [a `return` statement][LuaFormat::Return] or
//! [script with one or more variable assignments][LuaFormat::Script].
//!
//! [Maximum table depth limits are described in their own section](#maximum-table-depth).
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
//! | `table`   | [`LuaValue::Table`][]   | [`BTreeMap`][std::collections::BTreeMap], [`HashMap`][std::collections::HashMap], [`Vec<T>`], `struct` ([see note](#tables)) |
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
//! | Literal | [`LuaNumber`][] variant | [`f64`][] | [`i64`][] |
//! | ------- | :---------------------: | :-------: | :-------: |
//! | Decimal integer,<br>inside [`i64`][] range | [`Integer`][LuaNumber::Integer] | may lose precision | ✅ |
//! | Decimal integer,<br>outside [`i64`][] range | [`Float`][LuaNumber::Float]<br>will lose precision | will lose precision | ❌ |
//! | Hexadecimal integer | [`Integer`][LuaNumber::Integer] | may lose precision | ✅ |
//! | Decimal float | [`Float`][LuaNumber::Float] | ✅ | ❌ |
//! | Hexadecimal float[^wasm] | [`Float`][LuaNumber::Float] | ✅ | ❌ |
//! | `(0/0)` (NaN) | [`Float`][LuaNumber::Float] | ✅ | ❌ |
//!
//! [^wasm]: Not supported on WASM targets before v0.2.1.
//!
//! * A [`LuaNumber`][] field will follow Lua 5.4 semantics, which could be a
//!   [`i64`][LuaNumber::Integer] or [`f64`][LuaNumber::Float].
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
//! * Hexadecimal float literals with more than 16 hex digits will not parse, due to a limitation of
//!   the parsing library `serde_luaq` uses.
//!
//!   While Lua _accepts_ these values, `string.format('%q')` would never produce them.
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
//! Serde [`String`] fields can be used if the string literal evaluates to valid RFC 3629 UTF-8.
//! This is not guaranteed even if [the input data is `&str`][self::from_str], as Lua string escapes
//! may evaluate to binary values or invalid sequences (eg: `"\xC1\u{7FFFFFFF}"`).
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
//! This works everywhere but as a [flattened field's map value type](#flattening).
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
//! When deserialising a table as a `struct`, all keys must be written as valid
//! [RFC 3629 strings](#strings) or [Lua identifiers][LuaTableEntry::NameValue].
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
//! #### Flattening
//!
//! [`#[serde(flatten)]`][flatten] can be used with a map field:
//!
//! ```rust
//! # use std::collections::BTreeMap;
//! # use serde::Deserialize;
//! # use serde_luaq::{Error, LuaFormat, from_slice};
//! # fn main() -> Result<(), Error> {
//! #[derive(Deserialize, Debug, PartialEq)]
//! struct Flatten {
//!     version: i32,
//!     #[serde(flatten)]
//!     entries: BTreeMap<String, i64>,
//! }
//!
//! let lua = br#"{
//!     version = 1,
//!     example = 2,
//!     hello = 4,
//! }"#;
//!
//! assert_eq!(
//!     Flatten { version: 1, entries: BTreeMap::from([
//!         ("example".to_string(), 2),
//!         ("hello".to_string(), 4),
//!     ])},
//!     from_slice(lua, LuaFormat::Value, 16)?
//! );
//! # Ok(())
//! # }
//! ```
//!
//! If a flattened field's value is a table of only implicitly-keyed and/or numerically-keyed
//! entries, it can **only** go into a [`Vec`][] field (eg: `BTreeMap<String, Vec<i64>>`), and not a
//! nested map (eg: `BTreeMap<String, BTreeMap<i64, i64>>`).
//!
//! This is because Serde tries to handle these as an "any" type, and this library forces anything
//! that looks like an array or sparse array to be treated as an array.
//!
//! ### Enums
//!
//! When deserialising, `enum`s may be represented multiple ways:
//!
//! ```rust
//! enum E {
//!     /// `"Unit"` or `{["Unit"] = {}}`
//!     Unit,
//!
//!     /// `{["NewType"] = 1}`
//!     NewType(i64),
//!
//!     /// `{["Tuple"] = {1,2}}` or `{["Tuple"] = {[1]=1,[2]=2}}`
//!     Tuple(i64, i64),
//!
//!     /// `{["Struct"] = {["a"] = 1}`
//!     Struct { a: i64 },
//! }
//! ```
//!
//! [Like with tables](#tables-as-structs), if a variant's name is a valid Lua identifier, tables
//! may be keyed with an identifier instead of a string (eg: `{NewType = 1}`).
//!
//! ## Security
//!
//! While using Lua as a serialisation format is convenient to work with in Lua,
//! [its `load()`][load] and [`require()`][require] functions allow arbitrary code execution, so
//! aren't safe to use with untrusted inputs ([CWE-95][]). These risks are similar to using
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
//! If your program is ever sent untrusted Lua inputs, an attacker could insert some code which
//! could do anything to your program or the system it is running on. For example, this input would
//! cause Lua to read and return the contents of `/etc/passwd`:
//!
//! ```lua
//! (function() f=io.open('/etc/passwd');return f:read('a');end)()
//! ```
//!
//! Even if you attempted to sandbox the code with [`setfenv()`][setfenv] (Lua 5.1) or loaded it
//! into [an isolated environment][environments], an attacker could still make it consume a lot of
//! memory or CPU:
//!
//! ```lua
//! (function() x={};for a=1,100000000 do x[a]=a end;return x;end)()
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
//! read the same data structures, on a [`LuaValue`][] level (not Serde). If it doesn't, that's a
//! bug. :)
//!
//! ## Maximum table depth
//!
//! The `max_depth` argument controls how deeply nested a table can be before being rejected by
//! `serde_luaq`.
//!
//! Set this to the maximum depth of tables that you expect in your input data.
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
//! [to overflow its stack and crash][stackoverflow].
//!
//! What is "too high" depends on your platform and where you call `serde_luaq` in your program.
//!
//! Setting `max_depth` to `0` disables support for tables, _even empty tables_.
//!
//! </div>
//!
//! ## Memory usage
//!
//! Unless otherwise noted, all memory usage estimates assume a 64-bit target CPU.
//!
//! `serde_luaq` requires that the entire input fit in memory, and be less than [`usize::MAX`][]
//! bytes (4 GiB on 32-bit systems, 16 EiB on 64-bit systems). It is the _caller's_ responsibility
//! to enforce a reasonable input size limit for the system's available RAM.
//!
//! When deserialising a Lua data structure, the _minimum_ sizes of the [`LuaValue`][] and
//! [`LuaTableEntry`][] `enum`s are 32 and 16 bytes respectively. Values are checked at compile-time
//! on `aarch64`, `wasm32` and `x86_64` targets to prevent regressions.
//!
//! Heap-allocated variants of these `enum`s (those with [`Cow`][std::borrow::Cow] or [`Vec`][]
//! fields) use more memory.
//!
//! ### Large data structures
//!
//! At present, the highest-known memory usage per byte of input Lua is a table of deeply-nested
//! tables, which consumes up to 96 bytes of RAM for 2 bytes of input Lua (48&times;). This means a
//! 64 MiB input could use up to 3 GiB of RAM.
//!
//! Setting a maximum table depth of 2 could limit this to 56 bytes of RAM for 3 bytes of input Lua
//! (18.67&times;), or 1.167 GiB of RAM for a 64 MiB input.
//!
//! Lua uses similar amounts of memory for such data structures.
//!
//! When deserialising into your own data structures with Serde, be mindful that some Rust data
//! structures can use **significant amounts of memory** if you're not careful. Check out
//! [the Rust performance book][rust-perf] for tips.
//!
//! ### Large strings
//!
//! `serde_luaq` uses [`Cow`][std::borrow::Cow] to avoid owning strings whenever possible, borrowing
//! from the input buffer (for short strings that don't contain escape sequences, and long strings)
//! or `'static` (for empty strings and those containing a single, non-UTF-8 escape sequence) instead.
//!
//! Otherwise, it must be reassembled by copying it into an owned buffer.
//!
//! If the string consists entirely of escape sequences, the parser may temporarily use up to 24
//! bytes of memory per 2 bytes of input Lua (12&times;).
//!
//! The final, reassembled string will use up to 1 byte of memory for each byte of input Lua, plus
//! [`Vec`][]'s usual overheads (but doesn't allocate excess capacity).
//!
//! Unlike Lua, `serde_luaq` does not de-duplicate strings in a string table.
//!
//! ## Lua version compatibility
//!
//! `serde_luaq` targets syntax compatibility with Lua 5.4.
//!
//! As it does not execute Lua code, there are only a small number of compatibility issues with
//! older and other versions of Lua.
//!
//! ### Lua 5.3
//!
//! **Lua 5.3** over/underflows decimal integers that don't fit in a [`i64`][], rather than
//! coercing to [`f64`][].
//!
//! Hexadecimal integers over/underflow in both Lua 5.3 and 5.4.
//!
//! ### Lua 5.2
//!
//! **Lua 5.2 and earlier, and Luau** always use [`f64`][] for numbers, and do not have an integer
//! subtype.
//!
//! ### Lua 5.1 and earlier
//!
//! * `serde_luaq` only allows basic Latin letters in identifiers.
//!
//!   Lua 5.1 and earlier allows locale-dependent letters.
//!
//! * `serde_luaq` does not allow `goto` as an identifier name.
//!
//!   This is not a reserved keyword in Lua 5.1 and earlier.
//!
//! * `serde_luaq` allows empty statements in script mode.
//!
//!   [This is not allowed in Lua 5.1][empty-statements].
//!
//! ### Luau
//!
//! Like [Lua 5.2](#lua-5.2), **Luau** uses [`f64`][] for numbers.
//!
//! It also adds type annotations, binary integer literals, separators for all integer literals and
//! string interpolation.
//!
//! None of these features are supported by `serde_luaq`.
//!
//! ### Ravi
//!
//! **Ravi** adds type annotations and some other language features, which aren't supported by
//! `serde_luaq`.
//!
//! [comma]: https://github.com/lua/lua/blob/104b0fc7008b1f6b7d818985fbbad05cd37ee654/testes/literals.lua#L298-L300
//! [CWE-95]: https://cwe.mitre.org/data/definitions/95.html
//! [empty-statements]: https://www.lua.org/manual/5.1/manual.html#2.4.1
//! [environments]: https://www.lua.org/manual/5.4/manual.html#2.2
//! [flatten]: https://serde.rs/attr-flatten.html
//! [format]: https://www.lua.org/manual/5.4/manual.html#pdf-string.format
//! [`peg`]: https://docs.rs/peg/latest/peg/
//! [jseval]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/eval
//! [jsonparse]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON/parse
//! [load]: https://www.lua.org/manual/5.4/manual.html#pdf-load
//! [pil12.1.1]: https://www.lua.org/pil/12.1.1.html
//! [require]: https://www.lua.org/manual/5.4/manual.html#pdf-require
//! [rust-perf]: https://nnethercote.github.io/perf-book/type-sizes.html
//! [Serde]: https://serde.rs/
//! [serde_bytes]: https://docs.rs/serde_bytes/latest/serde_bytes/
//! [serde-num-keys]: https://github.com/serde-rs/serde/issues/2358
//! [setfenv]: https://www.lua.org/manual/5.1/manual.html#pdf-setfenv
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
