mod common;

use crate::common::{check, should_error, MAX_DEPTH};
use serde_luaq::{lua_value, script, LuaTableEntry, LuaValue};

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
wasm_bindgen_test_configure!(run_in_browser);

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn simple_table() -> Result {
    let data =
        br#"{["int"]=1,["seq"]={"a", "b", x3yz = 0x12, ["foo"] = "bar", [5] = 42, [0xa] = 3.14}}"#;

    let expected = LuaValue::Table(vec![
        LuaTableEntry::KeyValue(b"int".into(), LuaValue::integer(1)),
        LuaTableEntry::KeyValue(
            b"seq".into(),
            LuaValue::Table(vec![
                LuaTableEntry::Value(LuaValue::String(b"a".into())),
                LuaTableEntry::Value(LuaValue::String(b"b".into())),
                LuaTableEntry::NameValue("x3yz".into(), 0x12.into()),
                LuaTableEntry::KeyValue(b"foo".into(), b"bar".into()),
                LuaTableEntry::KeyValue(5.into(), 42.into()),
                LuaTableEntry::KeyValue(0xa.into(), 3.14.into()),
            ]),
        ),
    ]);

    let actual = lua_value(data, MAX_DEPTH)?;
    assert_eq!(expected, actual);

    // Table containing newlines and whitespace
    let data = br#"{
        ["int"]=1,
        ["seq"]={
            "a",
            "b";
            x3yz = 0x12,
            [ [[foo]]] = "bar",
            [5] = 42,
            [0xa] = 3.14,
        }
    }"#;

    let actual = lua_value(data, MAX_DEPTH)?;
    assert_eq!(expected, actual);

    // Table as script, containing newlines and whitespace
    let data = br#"
        int = 1
        seq = {
            "a",
            "b",
            x3yz = 0x12,
            [ [[foo]] ] = "bar",
            [5] = 42,
            [0xa] = 3.14
        }
    "#;

    let expected: Vec<(&str, LuaValue<'_>)> = vec![
        ("int", LuaValue::integer(1)),
        (
            "seq",
            LuaValue::Table(vec![
                LuaTableEntry::Value(LuaValue::String(b"a".into())),
                LuaTableEntry::Value(LuaValue::String(b"b".into())),
                LuaTableEntry::NameValue("x3yz".into(), 0x12.into()),
                LuaTableEntry::KeyValue(b"foo".into(), b"bar".into()),
                LuaTableEntry::KeyValue(5.into(), 42.into()),
                LuaTableEntry::KeyValue(0xa.into(), 3.14.into()),
            ]),
        ),
    ];
    let actual = script(data, MAX_DEPTH)?;
    assert_eq!(expected, actual);

    Ok(())
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn tables() {
    // Empty object
    let b = b"{}";
    check(b, LuaValue::Table(vec![]));

    // Empty object with space between braces
    let b = b"{ }";
    check(b, LuaValue::Table(vec![]));

    // Object containing nil
    let b = b"{nil}";
    check(b, LuaValue::Table(vec![LuaValue::Nil.into()]));

    // Example on https://www.lua.org/manual/5.4/manual.html#3.4.9, without function calls
    let b = b"{ [9999] = \"g\"; 'x', \"y\"; x = 1, 9999, [30] = 23; 45 }";
    check(
        b,
        LuaValue::Table(vec![
            LuaTableEntry::KeyValue(LuaValue::integer(9999), LuaValue::String(b"g".into())),
            LuaValue::String(b"x".into()).into(),
            LuaValue::String(b"y".into()).into(),
            LuaTableEntry::NameValue("x".into(), LuaValue::integer(1)),
            LuaTableEntry::Value(LuaValue::integer(9999)),
            LuaTableEntry::KeyValue(LuaValue::integer(30), LuaValue::integer(23)),
            LuaTableEntry::Value(LuaValue::integer(45)),
        ]),
    );
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn recursion() -> Result {
    let b = b"{}";
    assert!(lua_value(b, 0).is_err());
    assert_eq!(LuaValue::Table(vec![]), lua_value(b, 1)?);

    let b = b"{{{{{{{{{{{{{{{{}}}}}}}}}}}}}}}}";
    assert_eq!(b.len(), (MAX_DEPTH as usize) * 2);

    check(
        b,
        LuaValue::Table(vec![LuaTableEntry::Value(LuaValue::Table(vec![
            LuaTableEntry::Value(LuaValue::Table(vec![LuaTableEntry::Value(
                LuaValue::Table(vec![LuaTableEntry::Value(LuaValue::Table(vec![
                    LuaTableEntry::Value(LuaValue::Table(vec![LuaTableEntry::Value(
                        LuaValue::Table(vec![LuaTableEntry::Value(LuaValue::Table(vec![
                            LuaTableEntry::Value(LuaValue::Table(vec![LuaTableEntry::Value(
                                LuaValue::Table(vec![LuaTableEntry::Value(LuaValue::Table(vec![
                                    LuaTableEntry::Value(LuaValue::Table(vec![
                                        LuaTableEntry::Value(LuaValue::Table(vec![
                                            LuaTableEntry::Value(LuaValue::Table(vec![
                                                LuaTableEntry::Value(LuaValue::Table(vec![
                                                    LuaTableEntry::Value(LuaValue::Table(vec![])),
                                                ])),
                                            ])),
                                        ])),
                                    ])),
                                ]))]),
                            )])),
                        ]))]),
                    )])),
                ]))]),
            )])),
        ]))]),
    );

    let b = b"{{{{{{{{{{{{{{{{{}}}}}}}}}}}}}}}}}";
    assert_eq!(b.len(), (MAX_DEPTH as usize + 1) * 2);
    should_error(b);

    // Recursing heavily shouldn't crash.
    let mut b = Vec::with_capacity(65536);
    b.resize(32768, b'{');
    b.resize(65536, b'}');
    should_error(&b);

    Ok(())
}
