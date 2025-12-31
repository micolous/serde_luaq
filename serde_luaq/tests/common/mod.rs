#![allow(dead_code)]

use serde_luaq::{lua_value, return_statement, script, CompactFormatter, LuaValue};
use std::borrow::Borrow;

/// Maximum table depth for all tests. Our tests are very small!
pub const MAX_DEPTH: u16 = 16;

/// Parse a buffer of Lua code and expect no remaining value.
pub fn check<'a>(lua: &'_ [u8], expected: impl Borrow<LuaValue<'a>>) {
    let expected: &LuaValue<'a> = expected.borrow();
    let actual = lua_value(lua, MAX_DEPTH).expect(&format!(
        "parse error for Lua value: {}",
        lua.escape_ascii()
    ));

    if expected.is_nan() {
        assert!(actual.is_nan(), "lua: {}", lua.escape_ascii());
    } else {
        assert_eq!(&actual, expected, "lua: {}", lua.escape_ascii());
    }

    // Script
    let mut s = Vec::with_capacity(lua.len() + 4);
    s.extend_from_slice(b"a = ");
    s.extend_from_slice(lua);

    let (n, actual) = script(&s, MAX_DEPTH)
        .expect(&format!("parse error for Lua script: {}", s.escape_ascii()))
        .pop()
        .unwrap();
    assert_eq!("a", n);

    if expected.is_nan() {
        assert!(actual.is_nan(), "lua: {}", s.escape_ascii());
    } else {
        assert_eq!(&actual, expected, "lua: {}", s.escape_ascii());
    }

    // Return statement
    let mut s = Vec::with_capacity(lua.len() + 7);
    s.extend_from_slice(b"return ");
    s.extend_from_slice(lua);

    let actual = return_statement(&s, MAX_DEPTH)
        .expect(&format!("parse error for Lua return: {}", s.escape_ascii()));

    if expected.is_nan() {
        assert!(actual.is_nan(), "lua: {}", s.escape_ascii());
    } else {
        assert_eq!(&actual, expected, "lua: {}", s.escape_ascii());
    }

    // Return statement with extra whitespace
    let mut s = Vec::with_capacity(lua.len() + 7);
    s.extend_from_slice(b"return \n");
    s.extend_from_slice(lua);
    s.extend_from_slice(b"\n");

    let actual = return_statement(&s, MAX_DEPTH).expect(&format!(
        "parse error for Lua return with whitespace: {}",
        s.escape_ascii()
    ));

    if expected.is_nan() {
        assert!(actual.is_nan(), "lua: {}", s.escape_ascii());
    } else {
        assert_eq!(&actual, expected, "lua: {}", s.escape_ascii());
    }
}

pub fn should_error(lua: &'_ [u8]) {
    assert!(
        lua_value(lua, MAX_DEPTH).is_err(),
        "lua value: {}",
        lua.escape_ascii()
    );

    let mut r = Vec::with_capacity(lua.len() + 7);
    r.extend_from_slice(b"return ");
    r.extend_from_slice(lua);
    assert!(
        return_statement(lua, MAX_DEPTH).is_err(),
        "lua return: {}",
        lua.escape_ascii()
    );

    let mut s = Vec::with_capacity(lua.len() + 4);
    s.extend_from_slice(b"a = ");
    s.extend_from_slice(lua);

    assert!(
        script(lua, MAX_DEPTH).is_err(),
        "lua script: {}",
        lua.escape_ascii()
    );
}

pub fn check_format<'a>(value: impl Borrow<LuaValue<'a>>, expected: &[u8]) {
    let value: &LuaValue<'a> = value.borrow();

    let mut buf = Vec::with_capacity(128);
    let mut fmt = CompactFormatter;
    value.to_writer(&mut buf, &mut fmt).unwrap();
    assert_eq!(
        buf,
        expected,
        "\"{}\" != \"{}\"",
        buf.escape_ascii(),
        expected.escape_ascii(),
    );
}
