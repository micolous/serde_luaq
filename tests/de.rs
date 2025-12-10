//! Serde deserialisation tests.
mod common;
use crate::common::MAX_DEPTH;
use serde::Deserialize;
use serde_luaq::{from_slice, LuaFormat};
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
fn integers() {
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

    const RETURN_INTEGERS_DEC: &'static [u8] = include_bytes!("data/return/integers_dec.lua");
    assert_eq!(
        expected,
        from_slice(RETURN_INTEGERS_DEC, LuaFormat::Return, MAX_DEPTH).unwrap()
    );
    const RETURN_INTEGERS_HEX: &'static [u8] = include_bytes!("data/return/integers_hex.lua");
    assert_eq!(
        expected,
        from_slice(RETURN_INTEGERS_HEX, LuaFormat::Return, MAX_DEPTH).unwrap()
    );

    const SCRIPT_INTEGERS_DEC: &'static [u8] = include_bytes!("data/script/integers_dec.lua");
    assert_eq!(
        expected,
        from_slice(SCRIPT_INTEGERS_DEC, LuaFormat::Script, MAX_DEPTH).unwrap()
    );
    const SCRIPT_INTEGERS_HEX: &'static [u8] = include_bytes!("data/script/integers_hex.lua");
    assert_eq!(
        expected,
        from_slice(SCRIPT_INTEGERS_HEX, LuaFormat::Script, MAX_DEPTH).unwrap()
    );

    const VALUE_INTEGERS_DEC: &'static [u8] = include_bytes!("data/value/integers_dec.lua");
    assert_eq!(
        expected,
        from_slice(VALUE_INTEGERS_DEC, LuaFormat::Value, MAX_DEPTH).unwrap()
    );
    const VALUE_INTEGERS_HEX: &'static [u8] = include_bytes!("data/value/integers_hex.lua");
    assert_eq!(
        expected,
        from_slice(VALUE_INTEGERS_HEX, LuaFormat::Value, MAX_DEPTH).unwrap()
    );
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
    assert_eq!(
        expected,
        from_slice(b"{'hello', 'world'}", LuaFormat::Value, MAX_DEPTH)?
    );
    assert_eq!(
        expected,
        from_slice(
            b"{[1] = 'hello', [2] = 'world'}",
            LuaFormat::Value,
            MAX_DEPTH,
        )?
    );

    // Any gaps at the start of the array are filled with nil/None
    let expected = [None, Some("hello"), Some("world")];
    assert_eq!(
        expected,
        from_slice::<[Option<&str>; 3]>(b"{nil, 'hello', 'world'}", LuaFormat::Value, MAX_DEPTH)?
    );
    assert_eq!(
        expected,
        from_slice::<[Option<&str>; 3]>(
            b"{[2] = 'hello', [3] = 'world'}",
            LuaFormat::Value,
            MAX_DEPTH,
        )?
    );
    assert_eq!(
        expected,
        from_slice::<[Option<&str>; 3]>(
            b"{nil, nil, nil, [2] = 'hello', [3] = 'world'}",
            LuaFormat::Value,
            MAX_DEPTH,
        )?
    );
    assert_eq!(
        expected,
        from_slice::<[Option<&str>; 3]>(
            b"{[3] = 'world', [2] = 'hello'}",
            LuaFormat::Value,
            MAX_DEPTH,
        )?
    );

    // Gaps in the middle of the array are filled with nil/None
    let expected = [Some("hello"), None, Some("world")];
    assert_eq!(
        expected,
        from_slice::<[Option<&str>; 3]>(b"{'hello', nil, 'world'}", LuaFormat::Value, MAX_DEPTH)?
    );
    assert_eq!(
        expected,
        from_slice::<[Option<&str>; 3]>(b"{'hello', [3] = 'world'}", LuaFormat::Value, MAX_DEPTH)?
    );
    assert_eq!(
        expected,
        from_slice::<[Option<&str>; 3]>(
            b"{[1] = 'hello', [3] = 'world'}",
            LuaFormat::Value,
            MAX_DEPTH
        )?
    );
    assert_eq!(
        expected,
        from_slice::<[Option<&str>; 3]>(
            b"{nil, nil, nil, [1] = 'hello', [3] = 'world'}",
            LuaFormat::Value,
            MAX_DEPTH,
        )?
    );
    assert_eq!(
        expected,
        from_slice::<[Option<&str>; 3]>(
            b"{[3] = 'world', [1] = 'hello'}",
            LuaFormat::Value,
            MAX_DEPTH,
        )?
    );

    // Gaps at the end are not filled
    let expected = [Some("hello"), Some("world"), None];
    assert_eq!(
        expected,
        from_slice::<[Option<&str>; 3]>(b"{'hello', 'world', nil}", LuaFormat::Value, MAX_DEPTH)?
    );
    assert_eq!(
        expected,
        from_slice::<[Option<&str>; 3]>(
            b"{'hello', 'world', [3] = nil}",
            LuaFormat::Value,
            MAX_DEPTH,
        )?
    );
    assert_eq!(
        expected,
        from_slice::<[Option<&str>; 3]>(
            b"{'hello', [2] = 'world', [3] = nil}",
            LuaFormat::Value,
            MAX_DEPTH
        )?
    );
    assert_eq!(
        expected,
        from_slice::<[Option<&str>; 3]>(
            b"{[1] = 'hello', [2] = 'world', [3] = nil}",
            LuaFormat::Value,
            MAX_DEPTH,
        )?
    );

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

    Ok(())
}
