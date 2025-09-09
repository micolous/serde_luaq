mod common;
use serde_luaq::LuaValue;

use crate::common::check;

#[test]
fn booleans() {
    check(b"true", LuaValue::Boolean(true));
    check(b"false", LuaValue::Boolean(false));
}

#[test]
fn nil() {
    check(b"nil", LuaValue::Nil);
}
