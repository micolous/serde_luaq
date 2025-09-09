//! String literal tests
mod common;

use crate::common::check;
use serde_luaq::{lua_value, LuaTableEntry, LuaValue};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn basics() {
    // Empty string
    check(b"\"\"", LuaValue::String(b"".into()));
    check(b"''", LuaValue::String(b"".into()));

    check(b"\"hello world\"", LuaValue::String(b"hello world".into()));
    check(b"'hello world'", LuaValue::String(b"hello world".into()));
}

#[test]
fn long_string() {
    check(b"[[]]", LuaValue::String(b"".into()));
    check(b"[=[]=]", LuaValue::String(b"".into()));
    check(b"[==[]==]", LuaValue::String(b"".into()));
    check(b"[===[]===]", LuaValue::String(b"".into()));
    check(b"[====[]====]", LuaValue::String(b"".into()));
    check(b"[=====[]=====]", LuaValue::String(b"".into()));

    check(b"[[hello world]]", LuaValue::String(b"hello world".into()));
    check(
        b"[=[hello world]=]",
        LuaValue::String(b"hello world".into()),
    );
    check(
        b"[==[hello world]==]",
        LuaValue::String(b"hello world".into()),
    );
    check(
        b"[===[hello world]===]",
        LuaValue::String(b"hello world".into()),
    );
    check(
        b"[====[hello world]====]",
        LuaValue::String(b"hello world".into()),
    );
    check(
        b"[=====[hello world]=====]",
        LuaValue::String(b"hello world".into()),
    );

    // Newlines
    // Lua normalises these to the platform's newline character, but we retain these as-is because
    // it could otherwise affect unescaped binary data.
    check(
        b"[=[hello \n\n world]=]",
        LuaValue::String(b"hello \n\n world".into()),
    );
    check(
        b"[=[hello \r\r world]=]",
        LuaValue::String(b"hello \r\r world".into()),
    );
    check(
        b"[=[hello \r\n world]=]",
        LuaValue::String(b"hello \r\n world".into()),
    );
    check(
        b"[=[hello \n\r world]=]",
        LuaValue::String(b"hello \n\r world".into()),
    );

    // Escape sequences should not be interpreted.
    check(
        br"[[\0\001\n\r\023\t\u{65e5}]]",
        LuaValue::String(br"\0\001\n\r\023\t\u{65e5}".into()),
    );
    check(
        br"[==[\0\001\n\r\023\t]==]",
        LuaValue::String(br"\0\001\n\r\023\t".into()),
    );

    // Long brackets may only be ended with a bracket of the same level
    check(
        b"[=[hell[==[o]==] world]=]",
        LuaValue::String(b"hell[==[o]==] world".into()),
    );
    check(
        b"[=[hell[==[o[==[ world]=]",
        LuaValue::String(b"hell[==[o[==[ world".into()),
    );
    check(
        b"[=[hell[[o]] world]=]",
        LuaValue::String(b"hell[[o]] world".into()),
    );
    check(
        b"[[hell[=[o]=] world]]",
        LuaValue::String(b"hell[=[o]=] world".into()),
    );
    check(
        b"[[hell[=[o[==[ world]]",
        LuaValue::String(b"hell[=[o[==[ world".into()),
    );

    // Mix of short and long quotes
    check(
        b"[[hell\"o\" w'o'rld]]",
        LuaValue::String(b"hell\"o\" w'o'rld".into()),
    );
    check(
        b"\"hell[[o]] w[=[o]=]rld\"",
        LuaValue::String(b"hell[[o]] w[=[o]=]rld".into()),
    );

    // Multiple types of brackets in the same value
    check(
        b"{[[hello]],[=[world]=],'!',\"?\"}",
        LuaValue::Table(vec![
            LuaTableEntry::Value(LuaValue::String(b"hello".into())),
            LuaTableEntry::Value(LuaValue::String(b"world".into())),
            LuaTableEntry::Value(LuaValue::String(b"!".into())),
            LuaTableEntry::Value(LuaValue::String(b"?".into())),
        ]),
    );
}

#[test]
fn escapes() {
    // https://github.com/lua/tests/blob/26eebb47b6442996d89e298b99404cbf53468c4c/strings.lua#L152
    check(
        r#""\"ílo\"\n\\""#.as_bytes(),
        LuaValue::String("\"ílo\"\n\\".as_bytes().into()),
    );

    // Null bytes are allowed in strings
    check(b"\"\0\"", LuaValue::String(b"\0".into()));

    // ...so are invalid UTF-8 sequences
    check(b"\"\xFEedMe\"", LuaValue::String(b"\xFEedMe".into()));

    // ...and arbitrary binary data
    check(
        b"\"\0\x01\0023\x05\0009\"",
        LuaValue::String(b"\0\x01\0023\x05\0009".into()),
    );

    // escaped binary data
    check(
        b"\"\\0\\1\\02\\0023\\5\\0009\"",
        LuaValue::String(b"\0\x01\x02\x023\x05\09".into()),
    );

    // Backslash with line break.
    // Lua normalises these to the platform's newline character, but we retain these as-is because
    // it could otherwise affect unescaped binary data.
    check(
        b"\"hello\\\nworld\"",
        LuaValue::String(b"hello\nworld".into()),
    );

    check(
        b"\"hello\\\r\nworld\"",
        LuaValue::String(b"hello\r\nworld".into()),
    );

    check(
        b"\"hello\\\rworld\"",
        LuaValue::String(b"hello\rworld".into()),
    );

    check(
        b"\"\0\\rx\\r\\n\xFE\\0\\00\\000\\x00\\10\\010\\255\\xFF\\u{1f4a9}\"",
        LuaValue::String(b"\0\rx\r\n\xFE\0\0\0\0\x0A\x0A\xFF\xFF\xf0\x9f\x92\xa9".into()),
    );

    // \z escapes
    // https://github.com/lua/tests/blob/26eebb47b6442996d89e298b99404cbf53468c4c/literals.lua#L40
    check(b"\"abc\\z  \n   efg\"", LuaValue::String(b"abcefg".into()));
    check(b"\"abc\\zefg\"", LuaValue::String(b"abcefg".into()));

    check(b"\"abc\\z  \n\n\n\"", LuaValue::String(b"abc".into()));

    check(b"\"\\z  \n\t\x0c\x0b\n\"", LuaValue::String(b"".into()));
}

#[test]
fn unicode_escapes() {
    check(br"'\u{80}'", LuaValue::String(b"\xC2\x80".into()));
    check(
        br"'\u{10FFFF}'",
        LuaValue::String(b"\xF4\x8F\xBF\xBF".into()),
    );
    check(
        br"'\u{0000000000000010FFFF}'",
        LuaValue::String(b"\xF4\x8F\xBF\xBF".into()),
    );

    check(
        br"'\u{65E5}\u{672c}\u{8A9e}'",
        LuaValue::String("\u{65E5}\u{672C}\u{8a9E}".as_bytes().into()),
    );

    // Lua follows RFC 2279 UTF-8 (1998), so allows some sequences that were
    // later *disallowed* in RFC 3629 UTF-8 (2003).
    check(br"'\u{d800}'", LuaValue::String(b"\xED\xA0\x80".into()));
    check(br"'\u{dfff}'", LuaValue::String(b"\xED\xBF\xBF".into()));
    check(
        br"'\u{110000}'",
        LuaValue::String(b"\xF4\x90\x80\x80".into()),
    );

    check(
        br"'\u{70000000}'",
        LuaValue::String(b"\xFD\xB0\x80\x80\x80\x80".into()),
    );
    check(
        br"'\u{7fffffff}'",
        LuaValue::String(b"\xFD\xBF\xBF\xBF\xBF\xBF".into()),
    );

    // Strings should not be normalised
    check(
        b"'fran\xC3\xA7ais'",
        LuaValue::String(b"fran\xc3\xa7ais".into()),
    );
    check(
        br"'fran\xC3\xA7ais'",
        LuaValue::String(b"fran\xc3\xa7ais".into()),
    );
    check(
        br"'fran\u{E7}ais'",
        LuaValue::String(b"fran\xc3\xa7ais".into()),
    );

    check(
        b"'franc\xCC\xA7ais'",
        LuaValue::String(b"franc\xcc\xa7ais".into()),
    );
    check(
        br"'franc\xCC\xA7ais'",
        LuaValue::String(b"franc\xcc\xa7ais".into()),
    );
    check(
        br"'franc\u{327}ais'",
        LuaValue::String(b"franc\xcc\xa7ais".into()),
    );
}

#[test]
fn invalid_escapes() {
    assert!(lua_value(br"'\256'").is_err());
    assert!(lua_value(br"'\c'").is_err());
    assert!(lua_value(br"'\x'").is_err());
    assert!(lua_value(br"'\x0'").is_err());
    assert!(lua_value(br"'\xyz'").is_err());
    assert!(lua_value(br"'\u{80000000}'").is_err());
    assert!(lua_value(br"'\u{-80}'").is_err());
}

#[test]
fn borrows() -> Result {
    // Empty strings
    assert!(lua_value(b"[[]]")?.is_borrowed());
    assert!(lua_value(b"[=[]=]")?.is_borrowed());
    assert!(lua_value(b"''")?.is_borrowed());
    assert!(lua_value(b"\"\"")?.is_borrowed());

    // No escape sequences
    assert!(lua_value(b"[[hello]]")?.is_borrowed());
    assert!(lua_value(b"[=[hello]=]")?.is_borrowed());
    assert!(lua_value(b"'hello'")?.is_borrowed());
    assert!(lua_value(b"\"hello\"")?.is_borrowed());

    // Escapes are ignored for long bracket strings
    assert!(lua_value(br"[[hello\nworld]]")?.is_borrowed());
    assert!(lua_value(br"[=[hello\nworld]=]")?.is_borrowed());

    // Newline character should also be included
    assert!(lua_value(b"[[hello\nworld]]")?.is_borrowed());
    assert!(lua_value(b"[=[hello\nworld]=]")?.is_borrowed());

    // Strings containing _only_ an escape are borrowed
    assert!(lua_value(br"'\n'")?.is_borrowed());
    assert!(lua_value(b"\"\\n\"")?.is_borrowed());

    // Strings containing multiple escapes are owned
    assert!(!lua_value(br"'\r\n'")?.is_borrowed());
    assert!(!lua_value(b"\"\\r\\n\"")?.is_borrowed());

    // Strings containing escapes and non-escaped are owned
    assert!(!lua_value(br"'hello\n'")?.is_borrowed());
    assert!(!lua_value(b"\"hello\\n\"")?.is_borrowed());

    Ok(())
}
