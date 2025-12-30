//! Numeral literal tests
mod common;

use crate::common::{check, should_error};
use serde_luaq::LuaValue;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
wasm_bindgen_test_configure!(run_in_browser);

/// Decimal integer values
#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
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

    // Integer form of f64 bounds
    check(
        b"179769313486231570814527423731704356798070567525844996598917476803157260780028538760589558632766878171540458953514382464234321326889464182768467546703537516986049910576551282076245490090389328944075868508455133942304583236903222948165808559332123348274797826204144723168738177180919299881250404026184124858368",
        LuaValue::float(f64::MAX),
    );
    check(
        b"-179769313486231570814527423731704356798070567525844996598917476803157260780028538760589558632766878171540458953514382464234321326889464182768467546703537516986049910576551282076245490090389328944075868508455133942304583236903222948165808559332123348274797826204144723168738177180919299881250404026184124858368",
        LuaValue::float(f64::MIN),
    );

    // 2^1024 overflows
    check(
        b"179769313486231590772930519078902473361797697894230657273430081157732675805500963132708477322407536021120113879871393357658789768814416622492847430639474124377767893424865485276302219601246094119453082952085005768838150682342462881473913110540827237163350510684586298239947245938479716304835356329624224137216",
        LuaValue::float(f64::INFINITY),
    );
    check(
        b"-179769313486231590772930519078902473361797697894230657273430081157732675805500963132708477322407536021120113879871393357658789768814416622492847430639474124377767893424865485276302219601246094119453082952085005768838150682342462881473913110540827237163350510684586298239947245938479716304835356329624224137216",
        LuaValue::float(f64::NEG_INFINITY),
    );
}

/// Hex integer values
#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn hex_integers() {
    check(b"0xff", LuaValue::integer(0xff));
    check(b"0Xff", LuaValue::integer(0xff));
    check(b"+0xff", LuaValue::integer(0xff));
    check(b"+0Xff", LuaValue::integer(0xff));
    check(b"-0xff", LuaValue::integer(-0xff));
    check(b"-0Xff", LuaValue::integer(-0xff));
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
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
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

    // Maximum and minimum safe integer in f64
    check(b"9007199254740991.", LuaValue::float(9007199254740991.));
    check(b"-9007199254740991.", LuaValue::float(-9007199254740991.));

    // Expect loss of precision beyond those
    check(b"9007199254740992.", LuaValue::float(9007199254740992.));
    check(b"9007199254740993.", LuaValue::float(9007199254740992.));
    check(b"-9007199254740992.", LuaValue::float(-9007199254740992.));
    check(b"-9007199254740993.", LuaValue::float(-9007199254740992.));

    // f64 bounds
    check(b"1.7976931348623157e+308", LuaValue::float(f64::MAX));
    check(b"-1.7976931348623157e+308", LuaValue::float(f64::MIN));

    // Overflow f64
    check(b"1.8e+308", LuaValue::float(f64::INFINITY));
    check(b"-1.8e+308", LuaValue::float(f64::NEG_INFINITY));

    // We don't support locale-specific floats
    should_error(b"3,14");
    should_error(b"3,14e2");
    should_error(b".4,3");
}

/// Hex floats
#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn hex_floats() {
    check(b"0x1.", LuaValue::float(1.));
    check(b"0x1.0", LuaValue::float(1.));
    should_error(b"0x1.0p");
    check(b"0x1.0p0", LuaValue::float(1.));

    // lua-tests/math.lua, hex
    check(b"0E+1", LuaValue::float(0.));

    // We shouldn't be able to evaluate an expression, which could be confused
    // with a decimal exponent.
    should_error(b"0xE+1");
    should_error(b"0xE-1");
    should_error(b"0xe-");
    should_error(b"0xep-p");

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

    // https://github.com/lifthrasiir/hexf/issues/25
    // $ lua -e 'print(0x0.1E)'
    // 0.1171875
    check(b"0x.1E", LuaValue::float(0.1171875));
    check(b"0x0.1E", LuaValue::float(0.1171875));

    // https://docs.rs/hexfloat2/0.1.3/hexfloat2/
    // "too many hex digits" is allowed by Lua:
    // $ lua -e 'print(0x10000000000000000p20)'
    // 1.9342813113834e+25
    check(
        b"0x10000000000000000p20",
        LuaValue::float(1.9342813113834067e25),
    );

    // Lua prints both these as "1.0", but only the first is equal to 1.0.
    check(b"0x1.0000000000001p0", LuaValue::float(1.0000000000000002));
    check(b"0x1.00000000000001p0", LuaValue::float(1.));

    // We don't support locale-specific hex floats
    should_error(b"0x2,5");
    should_error(b"0xA,a");
    should_error(b"0xa,aP4");
}

/// Special floats
#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn special_floats() {
    check(b"(0/0)", LuaValue::float(f64::NAN));
    check(b"1e9999", LuaValue::float(f64::INFINITY));
    check(b"-1e9999", LuaValue::float(f64::NEG_INFINITY));
}
