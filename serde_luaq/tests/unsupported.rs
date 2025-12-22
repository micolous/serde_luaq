//! Tests for unsupported language features, which should _fail_ parsing.
//!
//! These should work in actual Lua.
mod common;

use crate::common::{should_error, MAX_DEPTH};
use serde_luaq::{lua_value, return_statement, script, LuaValue};

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
wasm_bindgen_test_configure!(run_in_browser);

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn arithmetic_add() {
    should_error(b"3 + 2\n");
    should_error(b"3+2\n");
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn arithmetic_sub() {
    should_error(b"3 - 2\n");
    should_error(b"3-2\n");
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn arithmetic_mul() {
    should_error(b"3 * 2\n");
    should_error(b"3*2\n");
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn arithmetic_div() {
    should_error(b"3 / 2\n");
    should_error(b"3/2\n");
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn arithmetic_fdiv() {
    should_error(b"3 // 2\n");
    should_error(b"3//2\n");
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn arithmetic_mod() {
    should_error(b"3 % 2\n");
    should_error(b"3%2\n");
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn arithmetic_exp() {
    should_error(b"3 ^ 2\n");
    should_error(b"3^2\n");
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn assignment() {
    assert!(return_statement(b"a = 3\nreturn a\n", MAX_DEPTH).is_err());
    assert!(return_statement(b"return a = 3\n", MAX_DEPTH).is_err());
    assert!(lua_value(b"a = 3\n", MAX_DEPTH).is_err());

    // But this should be valid for scripts.
    assert_eq!(
        vec![("a", LuaValue::integer(3))],
        script(b"a = 3\n", MAX_DEPTH).unwrap()
    );
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn bitwise_and() {
    should_error(b"3 & 2\n");
    should_error(b"3&2\n");
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn bitwise_or() {
    should_error(b"3 | 4\n");
    should_error(b"3|4\n");
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn bitwise_xor() {
    should_error(b"3 ~ 2\n");
    should_error(b"3~2\n");
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn bitwise_shl() {
    should_error(b"3 << 2\n");
    should_error(b"3<<2\n");
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn bitwise_shr() {
    should_error(b"3 >> 2\n");
    should_error(b"3>>2\n");
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn bitwise_not() {
    should_error(b"~3\n");
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn block_do() {
    assert!(script(b"a = 3\ndo\n  a = 4\nend\n", MAX_DEPTH).is_err());
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn comments() {
    assert!(script(b"-- line comment\na = 3\n", MAX_DEPTH).is_err());
    assert!(script(b"--[[ long comment ]]a = 3\n", MAX_DEPTH).is_err());
    assert!(script(b"--[[\nlong comment\n]]\na = 3\n", MAX_DEPTH).is_err());
    assert!(script(b"--[==[\nlonger comment\n]==]\na = 3\n", MAX_DEPTH).is_err());
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn coroutine() {
    should_error(
        b"coroutine.create(function (a)\n  coroutine.yield(a + 2)\n  return a + 10\nend)\n",
    );
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn functions() {
    assert!(script(b"function a()\n  return 3\nend\nb = a()\n", MAX_DEPTH).is_err());
    assert!(script(b"function a()\n  return 3\nend\nb=a()\n", MAX_DEPTH).is_err());
    assert!(script(b"return 3\n", MAX_DEPTH).is_err());
    assert!(lua_value(b"return 3\n", MAX_DEPTH).is_err());

    // But this should be valid for return statements.
    assert_eq!(
        LuaValue::integer(3),
        return_statement(b"return 3\n", MAX_DEPTH).unwrap()
    );
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn length_operator() {
    assert!(script(b"a = [1, 2, 3]\nb = #a\n", MAX_DEPTH).is_err());
    assert!(script(b"a=[1,2,3]\nb=#a\n", MAX_DEPTH).is_err());
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn local() {
    assert!(script(b"local a = 3\n", MAX_DEPTH).is_err());
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn logical_and() {
    should_error(b"true and true\n");
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn logical_or() {
    should_error(b"true or false\n");
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn logical_not() {
    should_error(b"b = not false\n");
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn parentheses() {
    should_error(b"(3)\n");
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn referencing() {
    assert!(script(b"a = 3\nb = a\n", MAX_DEPTH).is_err());
    assert!(script(b"a = {}\na.b = 3\n", MAX_DEPTH).is_err());
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn relational_eq() {
    assert!(script(b"a = 3\nb = a == 3\n", MAX_DEPTH).is_err());
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn relational_neq() {
    assert!(script(b"a = 3\nb = a ~= 1\n", MAX_DEPTH).is_err());
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn relational_lt() {
    assert!(script(b"a = 3\nb = a < 1\n", MAX_DEPTH).is_err());
    assert!(script(b"a = 3\nb=a<1\n", MAX_DEPTH).is_err());
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn relational_lte() {
    assert!(script(b"a = 3\nb = a <= 1\n", MAX_DEPTH).is_err());
    assert!(script(b"a = 3\nb=a<=1\n", MAX_DEPTH).is_err());
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn relational_gt() {
    assert!(script(b"a = 3\nb = a > 1\n", MAX_DEPTH).is_err());
    assert!(script(b"a = 3\nb=a>1\n", MAX_DEPTH).is_err());
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn relational_gte() {
    assert!(script(b"a = 3\nb = a >= 1\n", MAX_DEPTH).is_err());
    assert!(script(b"a = 3\nb=a>=1\n", MAX_DEPTH).is_err());
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn string_concat() {
    should_error(b"'hello' .. 'world'\n");
    should_error(b"'hello'..'world'\n");
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn vararg_assignments() {
    assert!(script(b"a, b = 'hello', 'world'\n", MAX_DEPTH).is_err());
    assert!(script(b"a,b='hello','world'\n", MAX_DEPTH).is_err());
}
