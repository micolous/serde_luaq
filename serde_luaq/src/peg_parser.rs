//! Peg-based Lua parser.
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use crate::strtod;
use crate::{wrapping_parse_int, LuaNumber, LuaTableEntry, LuaValue, LUA_KEYWORDS};
use std::{borrow::Cow, str::from_utf8};

const BELL: Cow<'static, [u8]> = Cow::Borrowed(b"\x07");
const BACKSPACE: Cow<'static, [u8]> = Cow::Borrowed(b"\x08");
const FORM_FEED: Cow<'static, [u8]> = Cow::Borrowed(b"\x0C");
const UNIX_LINEFEED: Cow<'static, [u8]> = Cow::Borrowed(b"\n");
const DOS_LINEFEED: Cow<'static, [u8]> = Cow::Borrowed(b"\r\n");
const ACORN_LINEFEED: Cow<'static, [u8]> = Cow::Borrowed(b"\n\r");
const CARRIAGE_RETURN: Cow<'static, [u8]> = Cow::Borrowed(b"\r");
const HORIZONTAL_TAB: Cow<'static, [u8]> = Cow::Borrowed(b"\t");
const VERTICAL_TAB: Cow<'static, [u8]> = Cow::Borrowed(b"\x0B");
const BACKSLASH: Cow<'static, [u8]> = Cow::Borrowed(br"\");
const QUOTATION_MARK: Cow<'static, [u8]> = Cow::Borrowed(b"\"");
const APOSTROPHE: Cow<'static, [u8]> = Cow::Borrowed(b"'");
const EMPTY: Cow<'static, [u8]> = Cow::Borrowed(b"");

/// Return a `'static` slice to a given byte.
///
/// `Cow<[u8]>` is the same size as `Vec<u8>` ([in Rust >= 1.76][0]), but this allows us to avoid a
/// bunch of 1-byte heap allocations.
///
/// However, it's still pretty stinky (`3 * usize`).
///
/// [0]: https://github.com/maciejhirsz/beef/issues/57#issuecomment-2730666291
#[inline]
fn slice_of_byte(i: u8) -> Cow<'static, [u8]> {
    const BYTES: [u8; 256] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
        48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70,
        71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93,
        94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112,
        113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130,
        131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148,
        149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166,
        167, 168, 169, 170, 171, 172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184,
        185, 186, 187, 188, 189, 190, 191, 192, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202,
        203, 204, 205, 206, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220,
        221, 222, 223, 224, 225, 226, 227, 228, 229, 230, 231, 232, 233, 234, 235, 236, 237, 238,
        239, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255,
    ];

    Cow::Borrowed(&BYTES[i as usize..][..1])
}

/// Merges zero or more string spans into a single string.
///
/// This tries to avoid copying where `s` is empty or contains exactly one span.
fn merge_spans<'a>(s: Vec<Cow<'a, [u8]>>) -> Cow<'a, [u8]> {
    if s.is_empty() {
        // Empty string
        return EMPTY;
    }

    if s.len() == 1 {
        // If there's only one span, return it directly, rather than
        // copying it.
        let mut s = s;
        return s.swap_remove(0);
    }

    let l: usize = s.iter().map(|c| c.len()).sum();
    let mut o = Vec::with_capacity(l);
    for i in s.into_iter() {
        match i {
            Cow::Borrowed(b) => o.extend_from_slice(b),
            Cow::Owned(mut v) => o.append(&mut v),
        }
    }

    Cow::Owned(o)
}

peg::parser! {
    pub grammar lua() for [u8] {
        rule identifier() -> &'input str
            = (
                i:$([ b'a'..=b'z' | b'A'..=b'Z' | b'_' ][ b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'0'..=b'9' ]*)
                {?
                    if LUA_KEYWORDS.binary_search(&i).is_ok() {
                        Err("identifier cannot be a reserved word")
                    } else {
                        // from_utf8 shouldn't error here
                        Ok(from_utf8(i).unwrap())
                    }
                }
            )
            / expected!("identifier")

        rule whitespace()
            = quiet!{[ b' ' | b'\n' | b'\t' | b'\r' | b'\x0b' | b'\x0c' ]}
            / expected!("whitespace")

        /// Match any number of whitespace characters (including zero).
        rule _ = whitespace()*

        /// Match at least one whitespace character.
        rule __ = whitespace()+

        /// Match any linebreak character sequence.
        rule linebreak()
            = "\r\n" / "\n\r" / "\r" / "\n"

        /// Match a decimal digit.
        rule digit() -> &'input [u8]
            = $(quiet!{[ b'0'..=b'9' ]})
            / expected!("digit")

        /// Match a hex digit.
        rule hex_digit() -> &'input [u8]
            = $(quiet!{[ b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' ]})
            / expected!("hex digit")

        /// Match at least 1 hex digit.
        rule hex_digits() -> &'input [u8]
            = $(quiet!{ hex_digit()+ })
            / expected!("hex digits")

        /// Parse a numeric value.
        rule numbers() -> LuaNumber
            = (
                "-1e9999" { LuaNumber::Float(f64::NEG_INFINITY) } /
                "1e9999" { LuaNumber::Float(f64::INFINITY) } /
                "(0/0)" { LuaNumber::Float(f64::NAN) } /

                (
                    n:$(
                        [ b'+' | b'-' ]?
                        (
                            // 3.14, 3., .14 with optional exponent
                            (
                                (
                                    (digit()+ [ b'.' ] digit()*) /
                                    ([ b'.' ] digit()+)
                                )
                                (
                                    [ b'e' | b'E' ]
                                    [ b'+' | b'-' ]?
                                    digit()+
                                )?
                            ) /
                            // 3e13, 3e2, 4e3 with mandatory exponent
                            (
                                digit()+
                                [ b'e' | b'E' ]
                                [ b'+' | b'-' ]?
                                digit()+
                            )
                        )
                    )
                    {?
                        // from_utf8 shouldn't error
                        let src = from_utf8(n).unwrap();
                        if let Ok(f) = str::parse(src) {
                            Ok(LuaNumber::Float(f))
                        } else {
                            Err("floating point parse error")
                        }
                    }
                ) /

                // 0xf0.0dp10, 0xf0.0d, 0xf00dp10
                (
                    n:$(
                        [ b'+' | b'-' ]?
                        ( "0x" / "0X" )
                        (
                            // 0xf0.(0d)(p[+-]10)
                            (
                                hex_digits()
                                "."
                                hex_digits()?
                                (
                                    [ b'P' | b'p' ]
                                    [ b'+' | b'-' ]?
                                    digit()+
                                )?
                            ) /
                            // 0x.0d(p[+-]10)
                            (
                                "."
                                hex_digits()
                                (
                                    [ b'P' | b'p' ]
                                    [ b'+' | b'-' ]?
                                    digit()+
                                )?
                            ) /
                            // 0x0dp[+-]10
                            (
                                hex_digits()
                                [ b'P' | b'p' ]
                                [ b'+' | b'-' ]?
                                digit()+
                            )
                        )
                    )
                    {?
                        // https://github.com/lua/lua/blob/f7439112a5469078ac4f444106242cf1c1d3fe8a/lstrlib.c#L1017
                        // https://github.com/lua/lua/blob/f7439112a5469078ac4f444106242cf1c1d3fe8a/lobject.c#L290
                        // strx2number: https://github.com/lua/lua/blob/f7439112a5469078ac4f444106242cf1c1d3fe8a/lobject.c#L227
                        // f64::from_str can't parse hex.
                        // Shell out to C's strtod, because that's easier.

                        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
                        {
                            // from_utf8 shouldn't error
                            let n = from_utf8(n).unwrap();

                            let Some(f) = strtod(n) else {
                                return Err("floating point parse error");
                            };
                            Ok(LuaNumber::Float(f))
                        }

                        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
                        {
                            return Err("parsing hex floating points is not supported on this platform")
                        }
                    }
                ) /

                // 0x1234
                (
                    sign:$([ b'+' | b'-' ]?)
                    "0"
                    [ b'X' | b'x' ]
                    n:$(hex_digits())
                    {?
                        let Some(i) = wrapping_parse_int(n, 16, sign != b"-") else {
                            return Err("hex integer parse error");
                        };
                        Ok(LuaNumber::Integer(i))
                    }
                ) /

                // 1234
                (
                    n:$(
                        [ b'+' | b'-' ]?
                        digit()+
                    )
                    {?
                        // from_utf8 shouldn't error
                        let src = from_utf8(n).unwrap();

                        if let Ok(n) = src.parse() {
                            Ok(LuaNumber::Integer(n))
                        } else if let Ok(f) = src.parse() {
                            // Coerce to float
                            Ok(LuaNumber::Float(f))
                        } else {
                            // Shouldn't get here
                            Err("decimal literal parse error")
                        }
                    }
                )
            )

        /// Parse a single escaped character, escaped newline sequence, or `\z`-sequence.
        ///
        /// The result will be `Owned` for `\u{XXXX}` escapes `>= 0x80`.
        rule escaped_char() -> Cow<'static, [u8]>
            = (
                // C-like escape sequences
                r"\a" { BELL } /
                r"\b" { BACKSPACE } /
                r"\f" { FORM_FEED } /
                r"\n" { UNIX_LINEFEED } /
                r"\r" { CARRIAGE_RETURN } /
                r"\t" { HORIZONTAL_TAB } /
                r"\v" { VERTICAL_TAB } /
                r"\\" { BACKSLASH } /
                "\\\"" { QUOTATION_MARK } /
                r"\'" { APOSTROPHE } /

                // backslash followed by linebreak -> newline in string
                "\\\r\n" { DOS_LINEFEED } /
                "\\\n\r" { ACORN_LINEFEED } /
                "\\\n" { UNIX_LINEFEED } /
                "\\\r" { CARRIAGE_RETURN } /

                // \z skips all following whitespace characters, including line breaks
                r"\z" _ { EMPTY } /

                // \x hex escape sequences
                r"\x" x:$(hex_digit()*<2>) {
                    // from_utf8 shouldn't error
                    let x = from_utf8(x).unwrap();
                    // u8::from_str_radix shouldn't error either
                    slice_of_byte(u8::from_str_radix(x, 16).unwrap())
                } /

                // \123 decimal escape sequences
                r"\" x:$(digit()*<1,3>) {?
                    // from_utf8 shouldn't error
                    let x = from_utf8(x).unwrap();

                    if let Ok(x) = x.parse() {
                        Ok(slice_of_byte(x))
                    } else {
                        Err("decimal escape too large")
                    }
                } /

                // \u{1234} Unicode characters, hex value less than 2**31
                // Lua allows these values to be 0-padded to any length, and
                // follows RFC 2279 rather than RFC 3629 (which restricted
                // things).
                //
                // luaO_utf8esc(): https://github.com/lua/lua/blob/9a3940380a2a1540dc500593a6de0c1c5e6feb69/lobject.c#L386
                r"\u{" x:$(hex_digits()) "}" {?
                    // This never panics
                    let x = from_utf8(x).unwrap();

                    let mut codepoint = match u32::from_str_radix(x, 16) {
                        // Character can be represented as-is using a single byte
                        Ok(codepoint) if codepoint < 0x80 => return Ok(slice_of_byte(codepoint as u8)),

                        // https://github.com/lua/lua/blob/9a3940380a2a1540dc500593a6de0c1c5e6feb69/lobject.c#L388
                        Ok(codepoint) if codepoint <= 0x7FFFFFFF => { codepoint },

                        _ => return Err("UTF-8 value too large"),
                    };

                    // Encode value as RFC 2279 UTF-8.
                    // https://github.com/lua/lua/blob/9a3940380a2a1540dc500593a6de0c1c5e6feb69/lobject.c#L392
                    let mut mfb = 0x3f;
                    let mut buff = Vec::with_capacity(8);
                    let mut n = 1;
                    while (codepoint > mfb) {
                        buff.push((0x80 | (codepoint & 0x3f)) as u8);
                        codepoint >>= 6;
                        mfb >>= 1;
                    }
                    buff.push(((!mfb << 1) | codepoint) as u8);

                    buff.reverse();
                    Ok(Cow::Owned(buff))
                } /

                expected!("valid escape sequence")
            )

        /// Parses a span of characters in a double-quoted string.
        rule double_quoted_chars() -> Cow<'input, [u8]>
            = (
                c:$([^ b'"' | b'\\' | b'\r' | b'\n' ]+) { c.into() }
                / escaped_char()
            )

        /// Parses a span of characters in a single-quoted string.
        rule single_quoted_chars() -> Cow<'input, [u8]>
            = (
                c:$([^ b'\'' | b'\\' | b'\r' | b'\n' ]+) { c.into() }
                / escaped_char()
            )

        /// Parses a double-quoted string.
        rule double_quoted_string() -> Cow<'input, [u8]>
            = "\"" s:double_quoted_chars()* "\"" {
                merge_spans(s)
            }

        /// Parses a single-quoted string.
        rule single_quoted_string() -> Cow<'input, [u8]>
            = "'" s:single_quoted_chars()* "'" {
                merge_spans(s)
            }

        // TODO: find a way to make this work with arbitrary levels.
        rule longer_string(level: usize) -> Cow<'input, [u8]>
            =
                "[" "="*<{level}> "["
                linebreak()?
                v:$(
                    (
                        !("]" "="*<{level}> "]")
                        [_]
                    )+
                )?
                "]" "="*<{level}> "]"
                { v.map(Cow::Borrowed).unwrap_or(EMPTY) }

        rule long_string() -> Cow<'input, [u8]>
            =
                "[["
                linebreak()?
                v:$(
                    (
                        !"]]"
                        [_]
                    )+
                )?
                "]]"
                { v.map(Cow::Borrowed).unwrap_or(EMPTY) }

        /// Parses a string.
        rule string() -> Cow<'input, [u8]>
            =
                single_quoted_string() /
                double_quoted_string() /
                long_string() /
                longer_string(1) /
                longer_string(2) /
                longer_string(3) /
                longer_string(4) /
                longer_string(5)

        /// Parse a bare Lua value expression as a [`LuaValue`].
        ///
        /// The value _may_ be preceeded or followed by whitespace.
        ///
        /// For more details about type mapping rules and parameters,
        /// [see the crate docs][crate#data-types].
        ///
        /// ## Example
        ///
        /// ```rust
        /// use serde_luaq::{lua_value, LuaValue};
        ///
        /// assert_eq!(LuaValue::Boolean(true), lua_value(b"true", 16).unwrap());
        /// assert_eq!(LuaValue::Boolean(false), lua_value(b"  false\r\n  ", 16).unwrap());
        /// ```
        ///
        /// For more information about Lua type conversion, see [`LuaValue`].
        pub rule lua_value(max_depth: u16) -> LuaValue<'input>
            = _ v:(
                "nil" { LuaValue::Nil } /
                "true" { LuaValue::Boolean(true) } /
                "false" { LuaValue::Boolean(false) } /
                n:numbers() { LuaValue::Number(n) } /
                s:string() { LuaValue::String(s) } /
                t:table(max_depth) { LuaValue::Table(t) } /
                expected!("Lua value")
            ) _ { v }

        rule table_entry(max_depth: u16) -> LuaTableEntry<'input>
            = _ v:(
                // "foo"
                val:lua_value(max_depth)
                {
                    LuaTableEntry::Value(val)
                } /

                // ["foo"]="bar"
                // [1234]="bar"
                "[" key:lua_value(max_depth) _ "]" _ "=" _ val:lua_value(max_depth)
                {
                    LuaTableEntry::KeyValue(key, val)
                } /

                // foo = "bar"
                key:identifier() _ "=" _ val:lua_value(max_depth)
                {
                    LuaTableEntry::NameValue(Cow::Borrowed(key), val)
                } /

                expected!("Lua table entry")
            ) _ { v }

        rule table_entries(max_depth: u16) -> Vec<LuaTableEntry<'input>>
            = entries:table_entry(max_depth) ** ([b',' | b';'])

        rule table(max_depth: u16) -> Vec<LuaTableEntry<'input>>
            =
                ("{" {?
                    // rust-peg doesn't have a stack limit; workaround based on
                    // https://github.com/kevinmehall/rust-peg/issues/282#issuecomment-2169784035
                    if max_depth == 0 {
                        Err("too deeply nested")
                    } else {
                        Ok(())
                    }
                })
                _
                e:table_entries(max_depth.saturating_sub(1))
                _
                // 3.4.9: [A table's] field list can have an optional trailing separator, as a
                // convenience for machine-generated code.
                [b',' | b';']?
                _
                "}" { e }

        rule assignment(max_depth: u16) -> (&'input str, LuaValue<'input>)
            = i:identifier() _ "=" _ v:lua_value(max_depth) { (i, v) }

        /// Parse a Lua script containing variable assignments into a [`Vec`] of
        /// `(&str, LuaValue)`.
        ///
        /// For more details about type mapping rules and parameters,
        /// [see the crate docs][crate#data-types].
        ///
        /// ## Example
        ///
        /// ```rust
        /// use serde_luaq::{script, LuaValue};
        ///
        /// assert_eq!(
        ///     vec![
        ///         ("hello", LuaValue::Boolean(true)),
        ///         ("goodbye", LuaValue::Boolean(false)),
        ///     ],
        ///     script(b"hello = true\ngoodbye = false", 16).unwrap()
        /// );
        /// ```
        ///
        /// For more information about Lua type conversion, see [`LuaValue`].
        pub rule script(max_depth: u16) -> Vec<(&'input str, LuaValue<'input>)>
            = (_ a:assignment(max_depth) _ (";" _)* { a })*

        /// Parse a Lua `return` stamement into a [`LuaValue`].
        ///
        /// For more details about type mapping rules and parameters,
        /// [see the crate docs][crate#data-types].
        ///
        /// ## Example
        ///
        /// ```rust
        /// use serde_luaq::{return_statement, LuaValue};
        ///
        /// assert_eq!(LuaValue::Boolean(true), return_statement(b"return true\n", 16).unwrap());
        /// ```
        ///
        /// For more information about Lua type conversion, see [`LuaValue`].
        pub rule return_statement(max_depth: u16) -> LuaValue<'input>
            = _ "return" __ v:lua_value(max_depth) _ { v }
    }
}
