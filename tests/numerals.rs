//! Numeral literal tests
mod common;

use crate::common::check;
use serde_luaq::{lua_value, LuaValue};

/// Decimal integer values
#[test]
fn decimal_integers() {
    // https://www.lua.org/manual/5.4/manual.html#3.1
    check(b"0", LuaValue::integer(0));
    check(b"+0", LuaValue::integer(0));
    check(b"-0", LuaValue::integer(-0));
    check(b"3", LuaValue::integer(3));
    check(b"+3", LuaValue::integer(3));
    check(b"-3", LuaValue::integer(-3));
    check(b"345", LuaValue::integer(345));
    check(b"+345", LuaValue::integer(345));
    check(b"-345", LuaValue::integer(-345));
    check(b"9223372036854775807", LuaValue::integer(i64::MAX));
    check(b"-9223372036854775808", LuaValue::integer(i64::MIN));

    // "a decimal integer numeral that overflows ... denotes a float"
    // Expected values account for floating point error
    check(
        b"9223372036854775808",
        LuaValue::float(9223372036854776000.),
    );
    check(
        b"-9223372036854775809",
        LuaValue::float(-9223372036854776000.),
    );
    check(
        b"18446744073709551615",
        LuaValue::float(18446744073709552000.),
    );
    check(
        b"-18446744073709551615",
        LuaValue::float(-18446744073709552000.),
    );
}

/// Hex integer values
#[test]
fn hex_integers() {
    check(b"0xff", LuaValue::integer(0xff));
    check(b"+0xff", LuaValue::integer(0xff));
    check(b"0xBEBADA", LuaValue::integer(0xBEBADA));
    check(b"+0xBEBADA", LuaValue::integer(0xBEBADA));
    check(b"0x7fffffffffffffff", LuaValue::integer(0x7fffffffffffffff));
    check(
        b"+0x7fffffffffffffff",
        LuaValue::integer(0x7fffffffffffffff),
    );
    check(
        b"-0x7fffffffffffffff",
        LuaValue::integer(-0x7fffffffffffffff),
    );
    check(
        b"-0x8000000000000000",
        LuaValue::integer(-0x8000000000000000),
    );

    // "Hexadecimal numerals with neither a radix point nor an exponent always denote an integer
    // value; if the value overflows, it wraps around to fit into a valid integer."
    check(
        b"0x8000000000000000",
        LuaValue::integer(0x8000000000000000_u64 as i64),
    );
    check(
        b"+0x8000000000000000",
        LuaValue::integer(0x8000000000000000_u64 as i64),
    );
    check(
        b"0xffffffffffffffff",
        LuaValue::integer(0xffffffffffffffff_u64 as i64),
    );
    check(
        b"+0xffffffffffffffff",
        LuaValue::integer(0xffffffffffffffff_u64 as i64),
    );
    check(
        b"-0xffffffffffffffff",
        LuaValue::integer(-0xffffffffffffffff_i128 as i64),
    );
    check(
        b"0x123456789abcdef01",
        LuaValue::integer(0x123456789abcdef01_u128 as i64),
    );
    check(
        b"+0x123456789abcdef01",
        LuaValue::integer(0x123456789abcdef01_u128 as i64),
    );
    check(
        b"-0x123456789abcdef01",
        LuaValue::integer(-0x123456789abcdef01_i128 as i64),
    );
}

/// Decimal floats
#[test]
fn decimal_floats() {
    check(b"0.0", LuaValue::float(0.0));
    check(b"-0.0", LuaValue::float(-0.0));
    check(b"3.0", LuaValue::float(3.0));
    check(b"-3.0", LuaValue::float(-3.0));
    check(b"3.1416", LuaValue::float(3.1416));
    check(b"-3.1416", LuaValue::float(-3.1416));
    check(b"314.16e-2", LuaValue::float(314.16e-2));
    check(b"-314.16e-2", LuaValue::float(-314.16e-2));
    check(b"0.31416E1", LuaValue::float(0.31416E1));
    check(b"0.31416E+1", LuaValue::float(0.31416E1));
    check(b"+0.31416E1", LuaValue::float(0.31416E1));
    check(b"+0.31416E+1", LuaValue::float(0.31416E1));
    check(b"-0.31416E1", LuaValue::float(-0.31416E1));
    check(b"34e1", LuaValue::float(34e1));
    check(b"-34e1", LuaValue::float(-34e1));
}

/// Hex floats
#[test]
fn hex_floats() {
    // lua-tests/math.lua, hex
    check(b"0E+1", LuaValue::float(0.));

    // We shouldn't be able to evaluate an expression, which could be confused
    // with a decimal exponent.
    assert!(lua_value(b"0xE+1").is_err());
    assert!(lua_value(b"0xE-1").is_err());

    check(b"0x1.fp10", LuaValue::float(1984.));

    // lua-tests/math.lua, floating hexes
    // let x = 2.3125; // 0x4002800000000000
    check(b"0x2.5", LuaValue::float(f64::from(0x25) / 16.)); // 2.3125
    check(b"+0x2.5", LuaValue::float(f64::from(0x25) / 16.));
    check(b"-0x2.5", LuaValue::float(f64::from(-0x25) / 16.));

    check(b"0x0p12", LuaValue::float(0.));
    check(b"-0x0p12", LuaValue::float(-0.));
    check(b"0x.0p-3", LuaValue::float(0.));
    check(b"+0x0.51p+8", LuaValue::float(0x51_i32.into()));
    check(b"0xA.a", LuaValue::float(10f64 + (10. / 16.)));
    check(b"0xa.aP4", LuaValue::float(0xAA.into()));
    check(b"0x.ABCDEFp+24", LuaValue::float(0xabcdef.into()));
}

/// Special floats
#[test]
fn special_floats() {
    check(b"(0/0)", LuaValue::float(f64::NAN));
    check(b"1e9999", LuaValue::float(f64::INFINITY));
    check(b"-1e9999", LuaValue::float(f64::NEG_INFINITY));
}
