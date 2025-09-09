//! Tests with Lua literals
#![allow(dead_code)]

use std::borrow::Borrow;

use serde_luaq::{lua_value, return_statement, script, LuaValue};

/// Parse a buffer of Lua code and expect no remaining value.
pub fn check<'a>(lua: &'_ [u8], expected: impl Borrow<LuaValue<'a>>) {
    let expected: &LuaValue<'a> = expected.borrow();
    let actual = lua_value(lua).unwrap();

    if expected.is_nan() {
        assert!(actual.is_nan(), "lua: {}", lua.escape_ascii());
    } else {
        assert_eq!(&actual, expected, "lua: {}", lua.escape_ascii());
    }

    let mut s = Vec::with_capacity(lua.len() + 4);
    s.extend_from_slice(b"a = ");
    s.extend_from_slice(lua);

    let (n, actual) = script(&s).unwrap().pop().unwrap();
    assert_eq!("a", n);

    if expected.is_nan() {
        assert!(actual.is_nan(), "lua: {}", s.escape_ascii());
    } else {
        assert_eq!(&actual, expected, "lua: {}", s.escape_ascii());
    }
}

pub fn should_error(lua: &'_ [u8]) {
    assert!(lua_value(lua).is_err(), "lua value: {}", lua.escape_ascii());

    let mut r = Vec::with_capacity(lua.len() + 7);
    r.extend_from_slice(b"return ");
    r.extend_from_slice(lua);
    assert!(
        return_statement(lua).is_err(),
        "lua return: {}",
        lua.escape_ascii()
    );

    let mut s = Vec::with_capacity(lua.len() + 4);
    s.extend_from_slice(b"a = ");
    s.extend_from_slice(lua);

    assert!(script(lua).is_err(), "lua script: {}", lua.escape_ascii());
}
