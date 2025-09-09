//! JSON conversion tests
use serde_json::json;
use serde_luaq::{
    from_json_value, lua_value, to_json_value, JsonConversionError, JsonConversionOptions,
    LuaTableEntry, LuaValue,
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const DEFAULT_OPTS: JsonConversionOptions = JsonConversionOptions {
    lossy_string: false,
};

#[test]
fn table_precedence() -> Result {
    // The last table entry takes precedence
    let expected = json!({"1": 2, "2": 4});
    let inputs: [&'static [u8]; 7] = [
        b"{[1] = 1, 2, [2] = 3, 4}",
        b"{1, [1] = 2, 3, [2] = 4}",
        b"{[1] = 1, [2] = 3, 2, 4}",
        b"{1, 3, [1] = 2, [2] = 4}",
        b"{1, 3, [1] = 2, [2] = 4}",
        b"{[1] = 1, 2, 3, [2] = 4}",
        b"{['1'] = 1, [1] = 2, [2] = 3, ['2'] = 4}",
    ];

    for input in inputs {
        assert_eq!(
            expected,
            to_json_value(lua_value(input)?, &DEFAULT_OPTS)?,
            "for input: {}",
            input.escape_ascii(),
        );
    }

    Ok(())
}

#[test]
fn table_coersion() -> Result {
    assert_eq!(
        json!([1, 2, 3, 4]),
        to_json_value(lua_value(b"{1, 2, 3, 4}")?, &DEFAULT_OPTS)?
    );
    assert_eq!(
        LuaValue::Table(vec![
            LuaTableEntry::Value(LuaValue::integer(1)),
            LuaTableEntry::Value(LuaValue::integer(2)),
            LuaTableEntry::Value(LuaValue::integer(3)),
            LuaTableEntry::Value(LuaValue::integer(4)),
        ]),
        from_json_value(json!([1, 2, 3, 4]))?,
    );

    assert_eq!(
        json!({"1": 1, "2": 2, "3": 3, "4": 4}),
        to_json_value(
            lua_value(b"{[1] = 1, [2] = 2, [3] = 3, [4] = 4}")?,
            &DEFAULT_OPTS
        )?
    );

    assert_eq!(
        json!({"1": 1, "2": 2, "3": 3, "4": 4}),
        to_json_value(
            lua_value(b"{['1'] = 1, ['2'] = 2, ['3'] = 3, ['4'] = 4}")?,
            &DEFAULT_OPTS
        )?
    );
    // JSON keys are always str, and all keys are not valid identifiers
    assert_eq!(
        LuaValue::Table(vec![
            LuaTableEntry::KeyValue(LuaValue::String(b"1".into()), LuaValue::integer(1)),
            LuaTableEntry::KeyValue(LuaValue::String(b"2".into()), LuaValue::integer(2)),
            LuaTableEntry::KeyValue(LuaValue::String(b"3".into()), LuaValue::integer(3)),
            LuaTableEntry::KeyValue(LuaValue::String(b"4".into()), LuaValue::integer(4)),
        ]),
        from_json_value(json!({"1": 1, "2": 2, "3": 3, "4": 4}))?,
    );

    assert_eq!(
        json!({"a": 1, "b": 2, "c": 3, "d": 4}),
        to_json_value(lua_value(b"{a = 1, b = 2, c = 3, d = 4}")?, &DEFAULT_OPTS)?
    );
    // Valid identifiers should be NameValue
    assert_eq!(
        LuaValue::Table(vec![
            LuaTableEntry::KeyValue(LuaValue::String(b"5".into()), LuaValue::integer(5)),
            LuaTableEntry::NameValue("a".into(), LuaValue::integer(1)),
            LuaTableEntry::NameValue("b".into(), LuaValue::integer(2)),
            LuaTableEntry::NameValue("c".into(), LuaValue::integer(3)),
            LuaTableEntry::NameValue("d".into(), LuaValue::integer(4)),
        ]),
        from_json_value(json!({"5": 5, "a": 1, "b": 2, "c": 3, "d": 4}))?,
    );

    assert_eq!(
        json!({"a": 1, "b": 2, "c": 3, "d": 4}),
        to_json_value(
            lua_value(b"{['a'] = 1, [\"b\"] = 2, [[[c]]] = 3, [[=[d]=]] = 4}")?,
            &DEFAULT_OPTS
        )?
    );

    // Mix => object
    assert_eq!(
        json!({"1": 1, "2": 2, "3": 3, "4": 4}),
        to_json_value(lua_value(b"{1, 2, 3, [4] = 4}")?, &DEFAULT_OPTS)?
    );
    assert_eq!(
        json!({"1": 1, "2": 2, "3": 3, "d": 4}),
        to_json_value(lua_value(b"{1, 2, 3, d = 4}")?, &DEFAULT_OPTS)?
    );
    assert_eq!(
        json!({"1": 1, "2": 2, "3": 3, "d": 4}),
        to_json_value(lua_value(b"{1, 2, [3] = 3, d = 4}")?, &DEFAULT_OPTS)?
    );

    // nil / bool => str
    assert_eq!(
        json!({"true": 1, "false": 2, "nil": 3}),
        to_json_value(
            lua_value(b"{[true] = 1, [false] = 2, [nil] = 3}")?,
            &DEFAULT_OPTS
        )?
    );

    // even when overwriting
    assert_eq!(
        json!({"true": 1, "false": 2, "nil": 3}),
        to_json_value(
            lua_value(
                b"{['true'] = 0, [true] = 1, [false] = 0, ['false'] = 2, [[[nil]]] = 0, [nil] = 3}"
            )?,
            &DEFAULT_OPTS
        )?
    );

    Ok(())
}

#[test]
fn disallowed_floats() -> Result {
    // JSON doesn't allow NaN, +Inf or -Inf
    assert_eq!(
        JsonConversionError::NaN,
        to_json_value(lua_value(b"(0/0)")?, &DEFAULT_OPTS).unwrap_err(),
    );

    assert_eq!(
        JsonConversionError::PositiveInfinity,
        to_json_value(lua_value(b"1e9999")?, &DEFAULT_OPTS).unwrap_err(),
    );

    assert_eq!(
        JsonConversionError::NegativeInfinity,
        to_json_value(lua_value(b"-1e9999")?, &DEFAULT_OPTS).unwrap_err(),
    );

    Ok(())
}

#[test]
fn floats() -> Result {
    assert_eq!(
        json!(2.3125),
        to_json_value(lua_value(b"0x2.5")?, &DEFAULT_OPTS)?
    );
    assert_eq!(LuaValue::float(2.3125), from_json_value(json!(2.3125))?,);

    assert_eq!(json!(2.0), to_json_value(lua_value(b"2.")?, &DEFAULT_OPTS)?);
    assert_eq!(LuaValue::float(2.0), from_json_value(json!(2.0))?);

    assert_eq!(
        json!(-2.0),
        to_json_value(lua_value(b"-2.")?, &DEFAULT_OPTS)?
    );
    assert_eq!(LuaValue::float(-2.), from_json_value(json!(-2.0))?);

    assert_eq!(
        json!(-0.0),
        to_json_value(lua_value(b"-0.")?, &DEFAULT_OPTS)?
    );
    assert_eq!(LuaValue::float(-0.), from_json_value(json!(-0.0))?);

    assert_eq!(json!(0.0), to_json_value(lua_value(b"0.")?, &DEFAULT_OPTS)?);
    assert_eq!(LuaValue::float(0.), from_json_value(json!(0.0))?);

    Ok(())
}

#[test]
fn ints() -> Result {
    assert_eq!(json!(2), to_json_value(lua_value(b"0x2")?, &DEFAULT_OPTS)?);
    assert_eq!(
        json!(65535),
        to_json_value(lua_value(b"0xffff")?, &DEFAULT_OPTS)?
    );

    Ok(())
}
