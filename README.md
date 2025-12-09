# `serde_luaq` &emsp; [![Latest Version]][crates.io] [![Docs version]][docs.rs]

[Latest Version]: https://img.shields.io/crates/v/serde_luaq.svg
[crates.io]: https://crates.io/crates/serde_luaq
[Docs version]: https://img.shields.io/docsrs/serde_luaq.svg
[docs.rs]: https://docs.rs/serde_luaq/

> [!NOTE]
> This library is still a work in progress, and there are no API stability guarantees.

`serde_luaq` is a library for deserialising (and eventually, serialising) simple, JSON-equivalent
data structures from Lua 5.4 source code, _without requiring Lua itself_ (unlike [`mlua`][mlua]).

The goal is to be able to read state from software (mostly games) which is serialised using
[Lua `%q` formatting][format] (and similar techniques) _without_ requiring arbitrary code execution.

This library consists of four parts:

- A [`LuaValue`][luavalue] `enum`, which describes Lua 5.4's basic data types (`nil`, boolean, string, number,
  table).

- A [`peg`][peg]-based parser for parsing a `&[u8]` (containing Lua) into a `LuaValue`.

- A [`serde`][serde]-based `Deserialize` implementation for converting a `LuaValue` into your own
  data types.

- _Optional_ lossy converter to and from `serde_json`'s `Value` type.

## Goal

For example, you could have a Lua script like this:

```lua
a = 1
b = {1, 2, 3}
c = {
    ["foo"] = "bar",
}
```

And define some a schema using Serde traits:

```rust
#[derive(Deserialize, PartialEq, Debug)]
struct ComplexType {
    foo: String,
}

#[derive(Deserialize, PartialEq, Debug)]
struct Test {
    a: u32,
    b: Vec<u32>,
    c: ComplexType,
}
```

Then deserialise it with:

```rust
use serde_luaq::{from_slice, LuaFormat};

let parsed: Test = serde_luaq::from_slice(
  input,
  LuaFormat::Script,
  /* maximum table depth */ 16,
).unwrap();

assert_eq!(parsed, Test {
  a: true,
  b: vec![1, 2, 3],
  c: ComplexType { foo: "bar".to_string() },
});
```

## Parser features

- [x] Input formats
  - [x] Bare Lua value expression, similar to JSON (`{["hello"] = "world"}`)
  - [x] Lua return statement (`return {["hello"] = "world"}`)
  - [x] Script with identifier assignments _only_ (`hello = "world"`)
- [ ] Serde (partial)
  - [x] Deserialising
  - [ ] Serialising
- [x] _Lossy_ `serde_json` interoperability
  - [x] `LuaValue` -> `serde_json::Value`
  - [x] `serde_json::Value` -> `LuaValue`

## Lua language features

This library aims to implement a _subset_ of Lua 5.4 that is equivalent to the subset of JavaScript
that a JSON parser would implement:

- [x] `nil`
- [x] Booleans (`true`, `false`)
- [x] [Numbers][lua3.1]
  - [x] [Integers][lua3.1]
    - [x] Decimal integers (`123`)
      - [x] Coercion to float for decimal numbers `< i64::MIN` or `> i64::MAX` ([Lua 5.4][lua8])
    - [x] Hexadecimal integer (`0xFF`)
      - [x] Wrapping large hexadecimal numbers to `i64`
  - [x] [Floats][lua3.1]
    - [x] Decimal floats with decimal point and optional exponent (`3.14`, `0.314e1`)
    - [x] Decimal floats with mandatory exponent (`3e14`)
    - [x] Hexadecimal floating points (`0x.ABCDEFp+24`) (*not supported on WASM*)
    - [x] Positive and negative infinity (`1e9999`, `-1e9999`)
    - [x] NaN (`(0/0)`)
- [x] [Strings][lua3.1]
  - [x] Strings in single quotes (`'`)
  - [x] Strings in double quotes (`"`)
  - [x] Strings in long brackets (`[[string]]`, `[==[string]==]`) (_up to 5 `=` deep_)
  - [x] Arbitrary 8-bit binary data inside strings (like `[u8]`)
  - [x] Escapes in quoted strings:
    - [x] C-like backslash-escapes (`abfnrtv\"'`)
    - [x] Escaped line breaks (`\\\n`, `\\\r`, `\\\r\n`, `\\\n\r`)
    - [x] `\z` whitespace span escapes (`str\z    ing` == `string`)
    - [x] Decimal escape sequences (`\1`, `\01`, `\001`)
    - [x] Hexadecimal escape sequences (`\x01`)
    - [x] UTF-8 escape sequences (`\u{1F4A9}`)
      - [x] Sequences allowed by RFC 2279 but not RFC 3629 (`\u{D800}`, `\u{7FFFFFFF}`)
- [x] [Tables][lua3.4.9]
  - [x] Key-values / expression keys (`{["foo"] = "bar"}`, `{[1234]="bar"}`)
  - [x] Name-values / identifier keys (`{foo = "bar"}`)
    - [x] Identifier validation (Lua 5.4-style)
  - [x] Values / implicit keys (`{"bar"}`)
  - [x] Mixed key types
  - [x] Recursion depth limits

This library is not designed to replace Lua, nor execute arbitrary Lua code, so these Lua features
are _intentionally unsupported_:

- Arithmetic operators (`+`, `-`, `*`, `/`...)
- Bitwise operators (`<<`, `>>`, `&`, `|`, `~`...)
- Blocks and visibility modifiers (`do ... end`, `local`)
- Comments
- Control structures (`if`, `break`, `for`, `goto`, `repeat`, `until`, `while`...)
- Function calls
- Function definitions
- Length operator (`#`)
- Logical operators (`and`, `or`, `not`)
- Newline character normalisation in strings (`\r\n` => `\n` on UNIX, `\n` => `\r\n` on Windows)
- Parentheses, except for `(0/0)` (NaN)
- Pointers (light userdata)
- Referencing other variables (`a = 10; b = a`)
- Relational operators (`==`, `~=`, `<`, `>`...)
- String concatenation (`"hello" .. " world"`)
- Threads and coroutines
- Updating other variables (`a = {}; a.b = 'foo'`)
- Userdata
- Vararg assignments and destructuring (`a, b = 1, 2`)

If you want to use these language features or otherwise need to run arbitrary Lua code, look at
something like [`mlua`][mlua], which links to `liblua`, and also provides `serde` bindings.

## Known users of Lua serialisation

- [SaveData][]: Love2D library, emits a `return` statement which is loaded by evaluating the string.

- [Balatro][] (`engine/string_packer.lua`) is a modified version of `SaveData` that also compresses
  with `deflate` for save games and settings (`.jkr` files).

- World of Warcraft: addon state, written to `WTF/{Account,SavedVariables}/**/*.lua` as scripts that
  set variables.

[Balatro]: https://www.playbalatro.com/
[format]: https://www.lua.org/manual/5.4/manual.html#pdf-string.format
[lua3.1]: https://www.lua.org/manual/5.4/manual.html#3.1
[lua3.4.9]: https://www.lua.org/manual/5.4/manual.html#3.4.9
[lua8]: https://www.lua.org/manual/5.4/manual.html#8
[luavalue]: https://docs.rs/serde_luaq/latest/serde_luaq/enum.LuaValue.html
[mlua]: https://github.com/mlua-rs/mlua
[peg]: https://docs.rs/peg/latest/peg/
[SaveData]: https://github.com/BroccoliRaab/SaveData
[serde]: https://serde.rs/
