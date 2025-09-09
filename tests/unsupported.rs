//! Tests for unsupported language features, which should _fail_ parsing.
mod common;

use crate::common::should_error;
use serde_luaq::{lua_value, return_statement, script, LuaValue};

#[test]
fn arithmetic_add() {
    should_error(b"3 + 2\n");
    should_error(b"3+2\n");
}

#[test]
fn arithmetic_sub() {
    should_error(b"3 - 2\n");
    should_error(b"3-2\n");
}

#[test]
fn arithmetic_mul() {
    should_error(b"3 * 2\n");
    should_error(b"3*2\n");
}

#[test]
fn arithmetic_div() {
    should_error(b"3 / 2\n");
    should_error(b"3/2\n");
}

#[test]
fn arithmetic_fdiv() {
    should_error(b"3 // 2\n");
    should_error(b"3//2\n");
}

#[test]
fn arithmetic_mod() {
    should_error(b"3 % 2\n");
    should_error(b"3%2\n");
}

#[test]
fn arithmetic_exp() {
    should_error(b"3 ^ 2\n");
    should_error(b"3^2\n");
}

#[test]
fn bitwise_and() {
    should_error(b"3 & 2\n");
    should_error(b"3&2\n");
}

#[test]
fn bitwise_or() {
    should_error(b"3 | 4\n");
    should_error(b"3|4\n");
}

#[test]
fn bitwise_xor() {
    should_error(b"3 ~ 2\n");
    should_error(b"3~2\n");
}

#[test]
fn bitwise_shl() {
    should_error(b"3 << 2\n");
    should_error(b"3<<2\n");
}

#[test]
fn bitwise_shr() {
    should_error(b"3 >> 2\n");
    should_error(b"3>>2\n");
}

#[test]
fn bitwise_not() {
    should_error(b"~3\n");
}

#[test]
fn block_do() {
    assert!(script(b"a = 3\ndo\n  a = 4\nend\n").is_err());
}

#[test]
fn coroutine() {
    should_error(
        b"coroutine.create(function (a)\n  coroutine.yield(a + 2)\n  return a + 10\nend)\n",
    );
}

#[test]
fn functions() {
    assert!(script(b"function a()\n  return 3\nend\nb = a()\n").is_err());
    assert!(script(b"function a()\n  return 3\nend\nb=a()\n").is_err());
    assert!(script(b"return 3\n").is_err());
    assert!(lua_value(b"return 3\n").is_err());

    // But this should be valid for return statements.
    assert_eq!(
        LuaValue::integer(3),
        return_statement(b"return 3\n").unwrap()
    );
}

#[test]
fn length_operator() {
    assert!(script(b"a = [1, 2, 3]\nb = #a\n").is_err());
    assert!(script(b"a=[1,2,3]\nb=#a\n").is_err());
}

#[test]
fn local() {
    assert!(script(b"local a = 3\n").is_err());
}

#[test]
fn logical_and() {
    should_error(b"true and true\n");
}

#[test]
fn logical_or() {
    should_error(b"true or false\n");
}

#[test]
fn logical_not() {
    should_error(b"b = not false\n");
}

#[test]
fn parentheses() {
    should_error(b"(3)\n");
}

#[test]
fn referencing() {
    assert!(script(b"a = 3\nb = a\n").is_err());
    assert!(script(b"a = {}\na.b = 3\n").is_err());
}

#[test]
fn relational_eq() {
    assert!(script(b"a = 3\nb = a == 3\n").is_err());
}

#[test]
fn relational_neq() {
    assert!(script(b"a = 3\nb = a ~= 1\n").is_err());
}

#[test]
fn relational_lt() {
    assert!(script(b"a = 3\nb = a < 1\n").is_err());
    assert!(script(b"a = 3\nb=a<1\n").is_err());
}

#[test]
fn relational_lte() {
    assert!(script(b"a = 3\nb = a <= 1\n").is_err());
    assert!(script(b"a = 3\nb=a<=1\n").is_err());
}

#[test]
fn relational_gt() {
    assert!(script(b"a = 3\nb = a > 1\n").is_err());
    assert!(script(b"a = 3\nb=a>1\n").is_err());
}

#[test]
fn relational_gte() {
    assert!(script(b"a = 3\nb = a >= 1\n").is_err());
    assert!(script(b"a = 3\nb=a>=1\n").is_err());
}

#[test]
fn string_concat() {
    should_error(b"'hello' .. 'world'\n");
    should_error(b"'hello'..'world'\n");
}

#[test]
fn vararg_assignments() {
    assert!(script(b"a, b = 'hello', 'world'\n").is_err());
    assert!(script(b"a,b='hello','world'\n").is_err());
}
