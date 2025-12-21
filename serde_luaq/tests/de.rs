//! Serde deserialisation tests.
mod common;
use crate::common::{check, MAX_DEPTH};
use serde::Deserialize;
use serde_luaq::{from_slice, LuaFormat, LuaNumber, LuaTableEntry, LuaValue};
use std::collections::BTreeMap;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
wasm_bindgen_test_configure!(run_in_browser);

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Deserialise a simple `struct`
#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn struct_simple() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Test {
        int: u32,
        seq: Vec<String>,
    }

    let expected = Test {
        int: 1,
        seq: vec!["a".to_owned(), "b".to_owned()],
    };

    // Test encoded as a table
    let j = br#"{["int"]=1,["seq"]={"a","b"}}"#;
    assert_eq!(
        expected,
        from_slice(j, LuaFormat::Value, MAX_DEPTH).unwrap()
    );

    // ...with newlines and extra whitespace
    let j = b"{\n\t[ \"int\" ]=1,\n\t[\"seq\"]={\n\t\t\"a\",\n\t\t\"b\"\n\t}\n}\n";
    assert_eq!(
        expected,
        from_slice(j, LuaFormat::Value, MAX_DEPTH).unwrap()
    );

    // Test encoded as a script
    let j = b"int = 1\nseq = {\"a\",\"b\"}";
    assert_eq!(
        expected,
        from_slice(j, LuaFormat::Script, MAX_DEPTH).unwrap()
    );

    // ...with a trailing newline
    let j = b"int = 1\nseq = {\"a\",\"b\"}\n";
    assert_eq!(
        expected,
        from_slice(j, LuaFormat::Script, MAX_DEPTH).unwrap()
    );

    // ...with DOS linefeeds
    let j = b"int = 1\r\nseq = {\"a\",\"b\"}";
    assert_eq!(
        expected,
        from_slice(j, LuaFormat::Script, MAX_DEPTH).unwrap()
    );

    // ...with DOS linefeeds and trailing new line
    let j = b"int = 1\r\nseq = {\"a\",\"b\"}\r\n";
    assert_eq!(
        expected,
        from_slice(j, LuaFormat::Script, MAX_DEPTH).unwrap()
    );
}

/// Deserialise a struct with a [`BTreeMap`] field
#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn btreemap_field() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Test {
        m: BTreeMap<i64, String>,
    }

    let j = br#"{["m"]={[1]="hello",[2]="goodbye"}}"#;
    let expected = Test {
        m: BTreeMap::from([(1, "hello".to_string()), (2, "goodbye".to_string())]),
    };
    assert_eq!(
        expected,
        from_slice(j, LuaFormat::Value, MAX_DEPTH).unwrap()
    );

    let j = br#"m = {[1]="hello",[2]="goodbye"}"#;
    assert_eq!(
        expected,
        from_slice(j, LuaFormat::Script, MAX_DEPTH).unwrap()
    );
}

/// Deseralise a [`BTreeMap`] directly
#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn btreemap_bare() {
    let lua_return = br#"return {[1]="hello",[2]="goodbye"}"#;
    let lua_value = br#"{[1]="hello",[2]="goodbye"}"#;
    let expected = BTreeMap::from([(1, "hello".to_string()), (2, "goodbye".to_string())]);
    assert_eq!(
        expected,
        from_slice(lua_return, LuaFormat::Return, MAX_DEPTH).unwrap()
    );
    assert_eq!(
        expected,
        from_slice(lua_value, LuaFormat::Value, MAX_DEPTH).unwrap()
    );

    let lua_return = br#"return {[true]="hello",[false]="goodbye"}"#;
    let lua_value = br#"{[true]="hello",[false]="goodbye"}"#;
    let expected = BTreeMap::from([(true, "hello".to_string()), (false, "goodbye".to_string())]);
    assert_eq!(
        expected,
        from_slice(lua_return, LuaFormat::Return, MAX_DEPTH).unwrap()
    );
    assert_eq!(
        expected,
        from_slice(lua_value, LuaFormat::Value, MAX_DEPTH).unwrap()
    );
}

/// Deserialise a [`BTreeMap`] with an `enum` key
#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn btreemap_enum_keys() {
    #[derive(Deserialize, PartialEq, Debug, PartialOrd, Eq, Ord)]
    enum Enum {
        A,
        B,
        C,
    }

    let lua_return = br#"return {["A"]="hello",["C"]="goodbye"}"#;
    let lua_script = b"A=\"hello\"\nC='goodbye'\n";
    let lua_value = br#"{["A"]="hello",["C"]="goodbye"}"#;

    let expected = BTreeMap::from([
        (Enum::A, "hello".to_string()),
        (Enum::C, "goodbye".to_string()),
    ]);

    assert_eq!(
        expected,
        from_slice(lua_return, LuaFormat::Return, MAX_DEPTH).unwrap()
    );
    assert_eq!(
        expected,
        from_slice(lua_script, LuaFormat::Script, MAX_DEPTH).unwrap()
    );
    assert_eq!(
        expected,
        from_slice(lua_value, LuaFormat::Value, MAX_DEPTH).unwrap()
    );
}

/// Deseraliase an `enum` with multiple variants.
#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn enum_variants() {
    #[derive(Deserialize, PartialEq, Debug)]
    enum E {
        Unit,
        Newtype(u32),
        Tuple(u32, u32),
        Struct { a: u32 },
    }

    // Can't represent bare Unit as script
    let lua_return = br#"return "Unit""#;
    let lua_value = br#""Unit""#;
    let expected = E::Unit;
    assert_eq!(
        expected,
        from_slice(lua_return, LuaFormat::Return, MAX_DEPTH).unwrap()
    );
    assert_eq!(
        expected,
        from_slice(lua_value, LuaFormat::Value, MAX_DEPTH).unwrap()
    );

    let lua_return = br#"return {["Newtype"]=1}"#;
    let lua_script = b"Newtype = 1\n";
    let lua_value = br#"{["Newtype"]=1}"#;
    let expected = E::Newtype(1);
    assert_eq!(
        expected,
        from_slice(lua_return, LuaFormat::Return, MAX_DEPTH).unwrap()
    );
    assert_eq!(
        expected,
        from_slice(lua_script, LuaFormat::Script, MAX_DEPTH).unwrap()
    );
    assert_eq!(
        expected,
        from_slice(lua_value, LuaFormat::Value, MAX_DEPTH).unwrap()
    );

    let lua_return = br#"return {["Tuple"]={1,2}}"#;
    let lua_script = b"Tuple = {1,2}\n";
    let lua_value = br#"{["Tuple"]={1,2}}"#;
    let expected = E::Tuple(1, 2);
    assert_eq!(
        expected,
        from_slice(lua_return, LuaFormat::Return, MAX_DEPTH).unwrap()
    );
    assert_eq!(
        expected,
        from_slice(lua_script, LuaFormat::Script, MAX_DEPTH).unwrap()
    );
    assert_eq!(
        expected,
        from_slice(lua_value, LuaFormat::Value, MAX_DEPTH).unwrap()
    );

    let lua_return = br#"return {["Struct"]={["a"]=1}}"#;
    let lua_script = br#"Struct = {["a"]=1}"#;
    let lua_value = br#"{["Struct"]={["a"]=1}}"#;
    let expected = E::Struct { a: 1 };
    assert_eq!(
        expected,
        from_slice(lua_return, LuaFormat::Return, MAX_DEPTH).unwrap()
    );
    assert_eq!(
        expected,
        from_slice(lua_script, LuaFormat::Script, MAX_DEPTH).unwrap()
    );
    assert_eq!(
        expected,
        from_slice(lua_value, LuaFormat::Value, MAX_DEPTH).unwrap()
    );
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn integers() -> Result {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Integers {
        i8: i8,
        i16: i16,
        i32: i32,
        i64: i64,
        u8: u8,
        u16: u16,
        u32: u32,
        u64: u64,
    }

    #[derive(Deserialize, PartialEq, Debug)]
    struct IntegerData {
        min: Integers,
        max: Integers,
    }

    #[derive(Deserialize, PartialEq, Debug)]
    struct IntegersNumber {
        i8: LuaNumber,
        i16: LuaNumber,
        i32: LuaNumber,
        i64: LuaNumber,
        u8: LuaNumber,
        u16: LuaNumber,
        u32: LuaNumber,
        u64: LuaNumber,
    }

    #[derive(Deserialize, PartialEq, Debug)]
    struct IntegerDataNumber {
        min: IntegersNumber,
        max: IntegersNumber,
    }

    let expected = IntegerData {
        min: Integers {
            i8: i8::MIN,
            i16: i16::MIN,
            i32: i32::MIN,
            i64: i64::MIN,
            u8: 0,
            u16: 0,
            u32: 0,
            u64: 0,
        },
        max: Integers {
            i8: i8::MAX,
            i16: i16::MAX,
            i32: i32::MAX,
            i64: i64::MAX,
            u8: u8::MAX,
            u16: u16::MAX,
            u32: u32::MAX,
            // Lua's integer is a wrapping i64
            u64: i64::MAX as u64,
        },
    };

    let expected_number = IntegerDataNumber {
        min: IntegersNumber {
            i8: i8::MIN.into(),
            i16: i16::MIN.into(),
            i32: i32::MIN.into(),
            i64: i64::MIN.into(),
            u8: 0.into(),
            u16: 0.into(),
            u32: 0.into(),
            u64: 0.into(),
        },
        max: IntegersNumber {
            i8: i8::MAX.into(),
            i16: i16::MAX.into(),
            i32: i32::MAX.into(),
            i64: i64::MAX.into(),
            u8: u8::MAX.into(),
            u16: u16::MAX.into(),
            u32: u32::MAX.into(),
            // Lua's integer is a wrapping i64
            u64: i64::MAX.into(),
        },
    };

    const RETURN_INTEGERS_DEC: &'static [u8] = include_bytes!("data/return/integers_dec.lua");
    assert_eq!(
        expected,
        from_slice(RETURN_INTEGERS_DEC, LuaFormat::Return, MAX_DEPTH)?
    );
    assert_eq!(
        expected_number,
        from_slice(RETURN_INTEGERS_DEC, LuaFormat::Return, MAX_DEPTH)?
    );

    const RETURN_INTEGERS_HEX: &'static [u8] = include_bytes!("data/return/integers_hex.lua");
    assert_eq!(
        expected,
        from_slice(RETURN_INTEGERS_HEX, LuaFormat::Return, MAX_DEPTH)?
    );
    assert_eq!(
        expected_number,
        from_slice(RETURN_INTEGERS_HEX, LuaFormat::Return, MAX_DEPTH)?
    );

    const SCRIPT_INTEGERS_DEC: &'static [u8] = include_bytes!("data/script/integers_dec.lua");
    assert_eq!(
        expected,
        from_slice(SCRIPT_INTEGERS_DEC, LuaFormat::Script, MAX_DEPTH)?
    );
    assert_eq!(
        expected_number,
        from_slice(SCRIPT_INTEGERS_DEC, LuaFormat::Script, MAX_DEPTH)?
    );

    const SCRIPT_INTEGERS_HEX: &'static [u8] = include_bytes!("data/script/integers_hex.lua");
    assert_eq!(
        expected,
        from_slice(SCRIPT_INTEGERS_HEX, LuaFormat::Script, MAX_DEPTH)?
    );
    assert_eq!(
        expected_number,
        from_slice(SCRIPT_INTEGERS_HEX, LuaFormat::Script, MAX_DEPTH)?
    );

    const VALUE_INTEGERS_DEC: &'static [u8] = include_bytes!("data/value/integers_dec.lua");
    assert_eq!(
        expected,
        from_slice(VALUE_INTEGERS_DEC, LuaFormat::Value, MAX_DEPTH)?
    );
    assert_eq!(
        expected_number,
        from_slice(VALUE_INTEGERS_DEC, LuaFormat::Value, MAX_DEPTH)?
    );

    const VALUE_INTEGERS_HEX: &'static [u8] = include_bytes!("data/value/integers_hex.lua");
    assert_eq!(
        expected,
        from_slice(VALUE_INTEGERS_HEX, LuaFormat::Value, MAX_DEPTH)?
    );
    assert_eq!(
        expected_number,
        from_slice(VALUE_INTEGERS_HEX, LuaFormat::Value, MAX_DEPTH)?
    );

    Ok(())
}

/// Hex integers for signed fields overflow as [i64][] only.
#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn hex_overflow() -> Result {
    // i8
    assert_eq!(
        -1,
        from_slice::<i8>(b"0xffffffffffffffff", LuaFormat::Value, MAX_DEPTH)?,
    );
    assert_eq!(
        i8::MIN,
        from_slice::<i8>(b"0xffffffffffffff80", LuaFormat::Value, MAX_DEPTH)?,
    );
    assert!(from_slice::<i8>(b"0x80", LuaFormat::Value, MAX_DEPTH).is_err());
    assert!(from_slice::<i8>(b"0xff", LuaFormat::Value, MAX_DEPTH).is_err());
    assert!(from_slice::<i8>(b"0xffffffffffffff7f", LuaFormat::Value, MAX_DEPTH).is_err());

    // i16
    assert_eq!(
        -1,
        from_slice::<i16>(b"0xffffffffffffffff", LuaFormat::Value, MAX_DEPTH)?,
    );
    assert_eq!(
        i16::MIN,
        from_slice::<i16>(b"0xffffffffffff8000", LuaFormat::Value, MAX_DEPTH)?,
    );
    assert!(from_slice::<i16>(b"0x8000", LuaFormat::Value, MAX_DEPTH).is_err());
    assert!(from_slice::<i16>(b"0xffff", LuaFormat::Value, MAX_DEPTH).is_err());
    assert!(from_slice::<i16>(b"0xffffffffffff7fff", LuaFormat::Value, MAX_DEPTH).is_err());

    // i32
    assert_eq!(
        -1,
        from_slice::<i32>(b"0xffffffffffffffff", LuaFormat::Value, MAX_DEPTH)?,
    );
    assert_eq!(
        i32::MIN,
        from_slice::<i32>(b"0xffffffff80000000", LuaFormat::Value, MAX_DEPTH)?,
    );
    assert!(from_slice::<i32>(b"0x80000000", LuaFormat::Value, MAX_DEPTH).is_err());
    assert!(from_slice::<i32>(b"0xffffffff", LuaFormat::Value, MAX_DEPTH).is_err());
    assert!(from_slice::<i32>(b"0xffffffff7fffffff", LuaFormat::Value, MAX_DEPTH).is_err());

    // i64
    assert_eq!(
        -1,
        from_slice::<i64>(b"0xffffffffffffffff", LuaFormat::Value, MAX_DEPTH)?,
    );
    assert_eq!(
        i64::MIN,
        from_slice::<i64>(b"0x8000000000000000", LuaFormat::Value, MAX_DEPTH)?,
    );

    Ok(())
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn floats() -> Result {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Floats {
        f32: f32,
        f64: f64,
    }

    let expected = Floats {
        f32: f32::MAX,
        f64: f64::MAX,
    };
    assert_eq!(
        expected,
        from_slice(
            b"{f32 = 3.4028235e38, f64 = 1.7976931348623157e308}",
            LuaFormat::Value,
            MAX_DEPTH,
        )?,
    );
    assert_eq!(
        expected,
        from_slice(
            b"{f32 = 340282349999999991754788743781432688640, f64 = 179769313486231570814527423731704356798070567525844996598917476803157260780028538760589558632766878171540458953514382464234321326889464182768467546703537516986049910576551282076245490090389328944075868508455133942304583236903222948165808559332123348274797826204144723168738177180919299881250404026184124858368}",
            LuaFormat::Value,
            MAX_DEPTH,
        )?,
    );

    let expected = Floats {
        f32: f32::MIN,
        f64: f64::MIN,
    };
    assert_eq!(
        expected,
        from_slice(
            b"{f32 = -3.4028235e38, f64 = -1.7976931348623157e308}",
            LuaFormat::Value,
            MAX_DEPTH,
        )?,
    );
    assert_eq!(
        expected,
        from_slice(
            b"{f32 = -340282349999999991754788743781432688640, f64 = -179769313486231570814527423731704356798070567525844996598917476803157260780028538760589558632766878171540458953514382464234321326889464182768467546703537516986049910576551282076245490090389328944075868508455133942304583236903222948165808559332123348274797826204144723168738177180919299881250404026184124858368}",
            LuaFormat::Value,
            MAX_DEPTH,
        )?,
    );

    let expected = Floats {
        f32: f32::INFINITY,
        f64: f64::INFINITY,
    };
    assert_eq!(
        expected,
        from_slice(
            b"{f32 = 3.5e38, f64 = 1.8e308}",
            LuaFormat::Value,
            MAX_DEPTH,
        )?,
    );
    assert_eq!(
        expected,
        from_slice(
            b"{f32 = 350000000000000000000000000000000000000, f64 = 179769313486231590772930519078902473361797697894230657273430081157732675805500963132708477322407536021120113879871393357658789768814416622492847430639474124377767893424865485276302219601246094119453082952085005768838150682342462881473913110540827237163350510684586298239947245938479716304835356329624224137216}",
            LuaFormat::Value,
            MAX_DEPTH,
        )?,
    );

    let expected = Floats {
        f32: f32::NEG_INFINITY,
        f64: f64::NEG_INFINITY,
    };
    assert_eq!(
        expected,
        from_slice(
            b"{f32 = -3.5e38, f64 = -1.8e308}",
            LuaFormat::Value,
            MAX_DEPTH,
        )?,
    );
    assert_eq!(
        expected,
        from_slice(
            b"{f32 = -350000000000000000000000000000000000000, f64 = -179769313486231590772930519078902473361797697894230657273430081157732675805500963132708477322407536021120113879871393357658789768814416622492847430639474124377767893424865485276302219601246094119453082952085005768838150682342462881473913110540827237163350510684586298239947245938479716304835356329624224137216}",
            LuaFormat::Value,
            MAX_DEPTH,
        )?,
    );

    // Large hexadecimal integers overflow as i64 before converted to float.
    let expected = Floats {
        f32: 0xffffffffffffffff_u64 as i64 as f32,
        f64: 0xffffffffffffffff_u64 as i64 as f64,
    };
    assert_eq!(expected, Floats { f32: -1., f64: -1. });
    assert_eq!(
        expected,
        from_slice(
            b"{f32 = 0xffffffffffffffff, f64 = 0xffffffffffffffff}",
            LuaFormat::Value,
            MAX_DEPTH,
        )?,
    );

    // Same with underflows
    let expected = Floats {
        f32: -0xffffffffffffffff_i128 as i64 as f32,
        f64: -0xffffffffffffffff_i128 as i64 as f64,
    };
    assert_eq!(expected, Floats { f32: 1., f64: 1. });
    assert_eq!(
        expected,
        from_slice(
            b"{f32 = -0xffffffffffffffff, f64 = -0xffffffffffffffff}",
            LuaFormat::Value,
            MAX_DEPTH,
        )?,
    );

    let a: Floats = from_slice(b"{f32=(0/0), f64=(0/0)}", LuaFormat::Value, MAX_DEPTH)?;
    assert!(a.f32.is_nan());
    assert!(a.f64.is_nan());

    Ok(())
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn booleans() -> Result {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Booleans {
        a: bool,
        b: Option<bool>,
    }

    let expected = Booleans { a: true, b: None };
    assert_eq!(
        expected,
        from_slice(b"{a = true}", LuaFormat::Value, MAX_DEPTH)?
    );
    assert_eq!(
        expected,
        from_slice(b"{a = true, b = nil}", LuaFormat::Value, MAX_DEPTH)?
    );
    assert_eq!(
        expected,
        from_slice(b"{['a'] = true}", LuaFormat::Value, MAX_DEPTH)?
    );
    assert_eq!(
        expected,
        from_slice(b"{['a'] = true, b = nil}", LuaFormat::Value, MAX_DEPTH)?
    );

    let expected = Booleans {
        a: false,
        b: Some(true),
    };
    assert_eq!(
        expected,
        from_slice(b"{a = false, b = true}", LuaFormat::Value, MAX_DEPTH)?
    );
    assert_eq!(
        expected,
        from_slice(b"{['a'] = false, b = true}", LuaFormat::Value, MAX_DEPTH)?
    );
    assert_eq!(
        expected,
        from_slice(b"{a = false, ['b'] = true}", LuaFormat::Value, MAX_DEPTH)?
    );
    assert_eq!(
        expected,
        from_slice(
            b"{['a'] = false, ['b'] = true}",
            LuaFormat::Value,
            MAX_DEPTH
        )?
    );

    let expected = Booleans {
        a: false,
        b: Some(false),
    };
    assert_eq!(
        expected,
        from_slice(b"{a = false, b = false}", LuaFormat::Value, MAX_DEPTH)?
    );
    assert_eq!(
        expected,
        from_slice(b"{a = false, ['b'] = false}", LuaFormat::Value, MAX_DEPTH)?
    );
    assert_eq!(
        expected,
        from_slice(
            b"{['a'] = false, ['b'] = false}",
            LuaFormat::Value,
            MAX_DEPTH
        )?
    );
    assert_eq!(
        expected,
        from_slice(b"{['a'] = false, b = false}", LuaFormat::Value, MAX_DEPTH)?
    );

    #[derive(Deserialize, PartialEq, Debug, Default)]
    #[serde(default)]
    struct Booleans2 {
        a: bool,
        b: Option<bool>,
    }
    let expected = Booleans2 { a: false, b: None };
    assert_eq!(expected, from_slice(b"{}", LuaFormat::Value, MAX_DEPTH)?);
    assert_eq!(
        expected,
        from_slice(b"{b = nil}", LuaFormat::Value, MAX_DEPTH)?
    );
    assert_eq!(
        expected,
        from_slice(b"{a = false}", LuaFormat::Value, MAX_DEPTH)?
    );
    assert_eq!(
        expected,
        from_slice(b"{a = false, b = nil}", LuaFormat::Value, MAX_DEPTH)?
    );

    Ok(())
}

/// Tests for Serde's field naming
#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn field_naming() -> Result {
    #[derive(Deserialize, PartialEq, Debug)]
    struct FieldNaming {
        foo: i64,

        #[serde(rename = "1")]
        one: i64,

        snake_case: i64,
    }

    let expected = FieldNaming {
        foo: 1,
        one: 2,
        snake_case: 3,
    };

    assert_eq!(
        expected,
        from_slice(
            b"{['foo'] = 1, ['1'] = 2, snake_case = 3}",
            LuaFormat::Value,
            MAX_DEPTH,
        )?
    );

    #[derive(Deserialize, PartialEq, Debug)]
    struct UnicodeFields {
        français: i16,
        español: i16,
        māori: i16,
    }

    let expected = UnicodeFields {
        français: 1,
        español: 2,
        māori: 3,
    };

    assert_eq!(
        expected,
        from_slice(
            "{['français'] = 1, [\"español\"] = 2, [ [[māori]]] = 3}".as_bytes(),
            LuaFormat::Value,
            MAX_DEPTH,
        )?
    );

    assert!(from_slice::<UnicodeFields>(
        "{français=1, español=2, māori=3}".as_bytes(),
        LuaFormat::Value,
        MAX_DEPTH,
    )
    .is_err());

    assert!(from_slice::<UnicodeFields>(
        "français=1\nespañol=2\nmāori=3\n".as_bytes(),
        LuaFormat::Script,
        MAX_DEPTH,
    )
    .is_err());

    // Numeric key support blocked on https://github.com/serde-rs/serde/issues/2358
    // This also means we can't use a table with implicitly-keyed entries and map it to a numeric
    // key.
    Ok(())
}

#[test]
fn strings() -> Result {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Strings {
        a: u8,
        #[serde(with = "serde_bytes")]
        b: Vec<u8>,
        c: char,
        d: String,
    }

    let expected = Strings {
        a: 64,
        b: b"hello".to_vec(),
        c: '@',
        d: "world".to_string(),
    };

    assert_eq!(
        expected,
        from_slice(
            b"{a = 64, b = 'hello', c = '@', d = 'world'}",
            LuaFormat::Value,
            MAX_DEPTH,
        )?
    );
    assert_eq!(
        expected,
        from_slice(
            br"{a = 0x40, b = {104, 101, 108, 108, 111}, c = '\u{40}', d = '\119\111\114\108\100'}",
            LuaFormat::Value,
            MAX_DEPTH,
        )?
    );

    let expected = Strings {
        a: 32,
        b: vec![0, 1, 2, 3, 4, 5, 0xC0, 0xE0, 0],
        c: '#',
        d: "\u{65E5}\u{672C}\u{8A9E}".to_string(),
    };
    assert_eq!(
        expected,
        from_slice(
            b"{a = 32, b = '\0\\1\\2\\3\\004\\5\\xC0\\xE0\\0', c = '#', d = '\\u{65E5}\\u{672C}\\u{8A9E}'}",
            LuaFormat::Value, MAX_DEPTH,
        )?
    );
    assert_eq!(
        expected,
        from_slice(
            b"{a = 0x20, b = '\0\x01\x02\x03\x04\x05\xC0\xE0\0', c = '\\35', d = '\xe6\x97\xa5\xe6\x9c\xac\xe8\xaa\x9e'}",
            LuaFormat::Value, MAX_DEPTH,
        )?
    );

    // Serde handling byte strings as String
    let expected = serde_luaq::Error::SerdeDeserialize(
        "invalid value: byte array, expected UTF8 string".to_string(),
    );

    // RFC 2279 escapes
    assert_eq!(
        expected,
        from_slice::<String>(b"'\\u{d800}'", LuaFormat::Value, MAX_DEPTH).unwrap_err(),
    );
    assert_eq!(
        expected,
        from_slice::<String>(b"'\\u{7FFFFFFF}'", LuaFormat::Value, MAX_DEPTH).unwrap_err(),
    );

    // Binary data
    assert_eq!(
        expected,
        from_slice::<String>(b"'\xC0\xE0'", LuaFormat::Value, MAX_DEPTH).unwrap_err(),
    );

    // Escaped binary data
    assert_eq!(
        expected,
        from_slice::<String>(b"'\\xC0\\xE0'", LuaFormat::Value, MAX_DEPTH).unwrap_err(),
    );

    Ok(())
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn arrays() -> Result {
    let expected = ("hello", "world");
    let expected_a = ["hello", "world"];
    let expected_v = vec!["hello", "world"];
    let expected_m = BTreeMap::from([(1, "hello"), (2, "world")]);
    for b in [
        b"{'hello', 'world'}".as_slice(),
        b"{[1] = 'hello', [2] = 'world'}",
    ] {
        assert_eq!(
            expected,
            from_slice(b, LuaFormat::Value, MAX_DEPTH)?,
            "{}",
            b.escape_ascii()
        );
        assert_eq!(
            expected_a,
            from_slice::<[&str; 2]>(b, LuaFormat::Value, MAX_DEPTH)?,
            "{}",
            b.escape_ascii()
        );
        assert_eq!(
            expected_v,
            from_slice::<Vec<&str>>(b, LuaFormat::Value, MAX_DEPTH)?,
            "{}",
            b.escape_ascii()
        );
        assert_eq!(
            expected_m,
            from_slice(b, LuaFormat::Value, MAX_DEPTH)?,
            "{}",
            b.escape_ascii()
        );
    }

    // Any gaps at the start of the array are filled with nil/None
    let expected = vec![None, Some("hello"), Some("world")];
    for b in [
        b"{nil, 'hello', 'world'}".as_slice(),
        b"{nil, nil, nil, [2] = 'hello', [3] = 'world'}",
        b"{[2] = 'hello', [3] = 'world'}",
        b"{[3] = 'world', [2] = 'hello'}",
    ] {
        assert_eq!(
            expected,
            from_slice::<[Option<&str>; 3]>(b, LuaFormat::Value, MAX_DEPTH)?,
            "{}",
            b.escape_ascii()
        );
        assert_eq!(
            expected,
            from_slice::<Vec<Option<&str>>>(b, LuaFormat::Value, MAX_DEPTH)?,
            "{}",
            b.escape_ascii()
        );
    }

    // But putting a sequence into a map only fills explicit values, even when numerically keyed
    let expected_m = BTreeMap::from([(1, None), (2, Some("hello")), (3, Some("world"))]);
    for b in [
        b"{nil, 'hello', 'world'}".as_slice(),
        b"{nil, nil, nil, [2] = 'hello', [3] = 'world'}",
    ] {
        assert_eq!(
            expected_m,
            from_slice(b, LuaFormat::Value, MAX_DEPTH)?,
            "{}",
            b.escape_ascii()
        );
    }

    let expected_m = BTreeMap::from([(2, Some("hello")), (3, Some("world"))]);
    let expected_m2 = BTreeMap::from([(2, "hello"), (3, "world")]);
    for b in [
        b"{[2] = 'hello', [3] = 'world'}".as_slice(),
        b"{[3] = 'world', [2] = 'hello'}",
    ] {
        assert_eq!(
            expected_m,
            from_slice(b, LuaFormat::Value, MAX_DEPTH)?,
            "{}",
            b.escape_ascii()
        );
        assert_eq!(
            expected_m2,
            from_slice(b, LuaFormat::Value, MAX_DEPTH)?,
            "{}",
            b.escape_ascii()
        );
    }

    // Gaps in the middle of the array are filled with nil/None
    let expected = vec![Some("hello"), None, Some("world")];
    for b in [
        b"{'hello', nil, 'world'}".as_slice(),
        b"{nil, nil, nil, [1] = 'hello', [3] = 'world'}",
        b"{[1] = 'hello', [3] = 'world'}",
        b"{[3] = 'world', [1] = 'hello'}",
    ] {
        assert_eq!(
            expected,
            from_slice::<[Option<&str>; 3]>(b, LuaFormat::Value, MAX_DEPTH)?,
            "{}",
            b.escape_ascii()
        );
        assert_eq!(
            expected,
            from_slice::<Vec<Option<&str>>>(b, LuaFormat::Value, MAX_DEPTH)?,
            "{}",
            b.escape_ascii()
        );
    }

    // Gaps at the end are not filled unless explicitly provided
    let expected = vec![Some("hello"), Some("world"), None];
    let expected_m = BTreeMap::from([(1, Some("hello")), (2, Some("world")), (3, None)]);
    for b in [
        b"{'hello', 'world', nil}".as_slice(),
        b"{'hello', 'world', [3] = nil}",
        b"{'hello', [2] = 'world', [3] = nil}",
        b"{[1] = 'hello', [2] = 'world', [3] = nil}",
    ] {
        assert_eq!(
            expected,
            from_slice::<[Option<&str>; 3]>(b, LuaFormat::Value, MAX_DEPTH)?,
            "{}",
            b.escape_ascii()
        );
        assert_eq!(
            expected,
            from_slice::<Vec<Option<&str>>>(b, LuaFormat::Value, MAX_DEPTH)?,
            "{}",
            b.escape_ascii()
        );
        assert_eq!(
            expected_m,
            from_slice(b, LuaFormat::Value, MAX_DEPTH)?,
            "{}",
            b.escape_ascii()
        );
    }

    assert!(
        from_slice::<[Option<&str>; 3]>(b"{'hello', 'world'}", LuaFormat::Value, MAX_DEPTH)
            .is_err()
    );
    assert!(from_slice::<[Option<&str>; 3]>(
        b"{[1] = 'hello', [2] = 'world'}",
        LuaFormat::Value,
        MAX_DEPTH,
    )
    .is_err());

    // Mix implicit keys with a Map<int, ...>
    let expected = BTreeMap::from([(1, Some("hello")), (2, Some("world")), (3, None)]);
    assert_eq!(
        expected,
        from_slice::<BTreeMap<i64, Option<&str>>>(
            b"{'hello', 'world', nil}",
            LuaFormat::Value,
            MAX_DEPTH,
        )?
    );
    assert_eq!(
        expected,
        from_slice(
            b"{'hello', 'world', [3] = nil}",
            LuaFormat::Value,
            MAX_DEPTH,
        )?
    );
    assert_eq!(
        expected,
        from_slice(
            b"{'hello', [2] = 'world', [3] = nil}",
            LuaFormat::Value,
            MAX_DEPTH,
        )?
    );
    assert_eq!(
        expected,
        from_slice(
            b"{[1] = 'hello', [2] = 'world', [3] = nil}",
            LuaFormat::Value,
            MAX_DEPTH,
        )?
    );

    // Explicit and implicit keys with a Vec, with nil-filling.
    let expected: Vec<Option<&str>> = vec![None, Some("hello"), None, Some("world"), None];
    assert_eq!(
        expected,
        from_slice::<Vec<_>>(
            b"{nil, 'hello', nil, 'world', nil}",
            LuaFormat::Value,
            MAX_DEPTH,
        )?
    );
    assert_eq!(
        expected,
        from_slice::<Vec<_>>(
            b"{[2] = 'hello', [4] = 'world', [5] = nil}",
            LuaFormat::Value,
            MAX_DEPTH,
        )?
    );

    // No values at all!
    let expected: Vec<Option<i8>> = vec![None, None, None];
    assert_eq!(
        expected,
        from_slice::<Vec<_>>(b"{[3] = nil}", LuaFormat::Value, MAX_DEPTH)?
    );
    assert_eq!(
        expected,
        from_slice::<Vec<_>>(b"{nil, nil, nil}", LuaFormat::Value, MAX_DEPTH)?
    );

    let expected: Vec<Option<LuaNumber>> = vec![
        Some(LuaNumber::Integer(1)),
        None,
        Some(LuaNumber::Float(2.0)),
    ];
    assert_eq!(
        expected,
        from_slice::<Vec<_>>(b"{[3] = 2.0, 1}", LuaFormat::Value, MAX_DEPTH)?
    );

    Ok(())
}

/// Tests for `#[serde(flatten)]`
///
/// `#[serde(flatten)]` forces serde into the `deserialize_any` path, even if it should use
/// something more specialised. So, we treat everything that quacks like an array (implicit-keyed
/// values, and anything with explicit, numeric-only keys) as a `Vec`, so it can go into that type.
///
/// Unfortunately this breaks map-like usage of tables in the values of flattened types.
#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn flatten() -> Result {
    #[derive(Deserialize, Debug, PartialEq)]
    struct FlattenVecValue {
        version: i32,
        #[serde(flatten)]
        entries: BTreeMap<String, Vec<B>>,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct FlattenMapValue {
        version: i32,
        #[serde(flatten)]
        entries: BTreeMap<String, BTreeMap<i64, B>>,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct UnflattenVecValue {
        version: i32,
        abcd: Vec<B>,
        efgh: Vec<B>,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct UnflattenMapValue {
        version: i32,
        abcd: BTreeMap<i64, B>,
        efgh: BTreeMap<i64, B>,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct B {
        a: i32,
        b: i32,
    }

    let lua = br#"{
        ["version"] = 1,
        ["abcd"] = {
            { a = 1, b = 2, },
            { ['a'] = 2, ['b'] = 4, },
        },
        ["efgh"] = {
            { ["a"] = 4, ["b"] = 8, },
        },
    }"#;

    // Explicit keys
    let lua2 = br#"{
        ["version"] = 1,
        ["abcd"] = {
            [1] = { a = 1, b = 2, },
            [2] = { ['a'] = 2, ['b'] = 4, },
        },
        ["efgh"] = {
            { ["a"] = 4, ["b"] = 8, },
        },
    }"#;

    let expected_vec = FlattenVecValue {
        version: 1,
        entries: BTreeMap::from([
            ("abcd".to_string(), vec![B { a: 1, b: 2 }, B { a: 2, b: 4 }]),
            ("efgh".to_string(), vec![B { a: 4, b: 8 }]),
        ]),
    };

    let _expected_map = FlattenMapValue {
        version: 1,
        entries: BTreeMap::from([
            (
                "abcd".to_string(),
                BTreeMap::from([(1, B { a: 1, b: 2 }), (2, B { a: 2, b: 4 })]),
            ),
            ("efgh".to_string(), BTreeMap::from([(1, B { a: 4, b: 8 })])),
        ]),
    };

    let expected_unflatten_vec = UnflattenVecValue {
        version: 1,
        abcd: vec![B { a: 1, b: 2 }, B { a: 2, b: 4 }],
        efgh: vec![B { a: 4, b: 8 }],
    };

    let expected_unflatten_map = UnflattenMapValue {
        version: 1,
        abcd: BTreeMap::from([(1, B { a: 1, b: 2 }), (2, B { a: 2, b: 4 })]),
        efgh: BTreeMap::from([(1, B { a: 4, b: 8 })]),
    };

    let expected_raw = LuaValue::Table(vec![
        LuaTableEntry::KeyValue(Box::new((
            LuaValue::String(b"version".into()),
            LuaValue::integer(1),
        ))),
        LuaTableEntry::KeyValue(Box::new((
            LuaValue::String(b"abcd".into()),
            LuaValue::Table(vec![
                LuaTableEntry::Value(Box::new(LuaValue::Table(vec![
                    LuaTableEntry::NameValue(Box::new(("a".into(), LuaValue::integer(1)))),
                    LuaTableEntry::NameValue(Box::new(("b".into(), LuaValue::integer(2)))),
                ]))),
                LuaTableEntry::Value(Box::new(LuaValue::Table(vec![
                    LuaTableEntry::KeyValue(Box::new((
                        LuaValue::String(b"a".into()),
                        LuaValue::integer(2),
                    ))),
                    LuaTableEntry::KeyValue(Box::new((
                        LuaValue::String(b"b".into()),
                        LuaValue::integer(4),
                    ))),
                ]))),
            ]),
        ))),
        LuaTableEntry::KeyValue(Box::new((
            LuaValue::String(b"efgh".into()),
            LuaValue::Table(vec![LuaTableEntry::Value(Box::new(LuaValue::Table(vec![
                LuaTableEntry::KeyValue(Box::new((
                    LuaValue::String(b"a".into()),
                    LuaValue::integer(4),
                ))),
                LuaTableEntry::KeyValue(Box::new((
                    LuaValue::String(b"b".into()),
                    LuaValue::integer(8),
                ))),
            ])))]),
        ))),
    ]);

    check(lua, expected_raw);

    assert_eq!(expected_vec, from_slice(lua, LuaFormat::Value, MAX_DEPTH)?);
    assert_eq!(expected_vec, from_slice(lua2, LuaFormat::Value, MAX_DEPTH)?);

    assert_eq!(
        expected_unflatten_vec,
        from_slice(lua, LuaFormat::Value, MAX_DEPTH)?
    );
    assert_eq!(
        expected_unflatten_vec,
        from_slice(lua2, LuaFormat::Value, MAX_DEPTH)?
    );

    assert_eq!(
        expected_unflatten_map,
        from_slice(lua, LuaFormat::Value, MAX_DEPTH)?
    );
    assert_eq!(
        expected_unflatten_map,
        from_slice(lua2, LuaFormat::Value, MAX_DEPTH)?
    );

    // FIXME: broken due to some issue
    // assert_eq!(expected_map, from_slice(lua, LuaFormat::Value, MAX_DEPTH)?);

    Ok(())
}

/// Flattened `enum` parsing quirks.
///
/// Tests based on <https://github.com/serde-rs/serde/issues/1894>, but isn't actually that bug.
#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn enum_parse_quirks() -> Result {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Root {
        #[serde(flatten)]
        choice: Choice,
    }

    #[allow(non_camel_case_types)]
    #[derive(Debug, Deserialize, PartialEq)]
    enum Choice {
        one { item: Item },
        two { item: Vec<Item> },
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Item;

    let a = br#"{
        two = {
            item = {
                {},
                {},
            }
        }
    }"#;

    assert_eq!(
        Choice::two {
            item: vec![Item, Item]
        },
        from_slice(a, LuaFormat::Value, MAX_DEPTH).unwrap()
    );
    assert_eq!(
        Root {
            choice: Choice::two {
                item: vec![Item, Item]
            }
        },
        from_slice(a, LuaFormat::Value, MAX_DEPTH).unwrap()
    );

    let b = br#"{
        one = { item = {} }
    }"#;
    assert_eq!(
        Choice::one { item: Item },
        from_slice(b, LuaFormat::Value, MAX_DEPTH).unwrap()
    );
    assert_eq!(
        Root {
            choice: Choice::one { item: Item }
        },
        from_slice(b, LuaFormat::Value, MAX_DEPTH).unwrap()
    );

    let c = br#"{{
        one = { item = {} }
    }}"#;

    assert!(from_slice::<Root>(c, LuaFormat::Value, MAX_DEPTH).is_err());
    assert!(from_slice::<Choice>(c, LuaFormat::Value, MAX_DEPTH).is_err());
    Ok(())
}
