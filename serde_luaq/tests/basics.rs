mod common;
use crate::common::{check, MAX_DEPTH};
use serde_luaq::{script, LuaValue};

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
wasm_bindgen_test_configure!(run_in_browser);

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn booleans() {
    check(b"true", LuaValue::Boolean(true));
    check(b"false", LuaValue::Boolean(false));
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn nil() {
    check(b"nil", LuaValue::Nil);
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn script_expressions() -> Result {
    let expected = vec![("x", LuaValue::integer(4)), ("y", LuaValue::integer(5))];

    // "Lua has no line terminators"
    // https://the-ravi-programming-language.readthedocs.io/en/latest/lua-introduction.html#lua-has-no-line-terminators
    assert_eq!(expected, script(b"x=4 y=5", MAX_DEPTH)?);
    assert_eq!(expected, script(b"x = 4 y = 5", MAX_DEPTH)?);
    assert_eq!(expected, script(b"x=4;y=5", MAX_DEPTH)?);
    assert_eq!(expected, script(b"x = 4;y = 5", MAX_DEPTH)?);

    // This would be invalid in Lua 5.1
    // https://www.lua.org/manual/5.1/manual.html#2.4.1
    assert_eq!(expected, script(b"x=4;;y=5", MAX_DEPTH)?);

    // Different newlines
    assert_eq!(expected, script(b"x=4\ry=5", MAX_DEPTH)?);
    assert_eq!(expected, script(b"x=4\ry=5\r", MAX_DEPTH)?);
    assert_eq!(expected, script(b"x=4\ny=5", MAX_DEPTH)?);
    assert_eq!(expected, script(b"x=4\ny=5\n", MAX_DEPTH)?);
    assert_eq!(expected, script(b"x=4\r\ny=5", MAX_DEPTH)?);
    assert_eq!(expected, script(b"x=4\r\ny=5\r\n", MAX_DEPTH)?);
    assert_eq!(expected, script(b"x=4\n\ry=5", MAX_DEPTH)?);
    assert_eq!(expected, script(b"x=4\n\ry=5\n\r", MAX_DEPTH)?);
    assert_eq!(expected, script(b"x=4\n  \n  \n\t  y=5", MAX_DEPTH)?);
    assert_eq!(
        expected,
        script(b"x=\n\n\n\n\r\n\n\r\n4\ny      =\n5", MAX_DEPTH)?
    );

    // syntax error near y
    assert!(script(b"x y = 5", MAX_DEPTH).is_err());

    // calls function x with args ('4')
    assert!(script(b"x '4' y = 5", MAX_DEPTH).is_err());

    // Creates a local variable x (which is set to `nil`), and sets the global y to 5.
    // Not supported because it uses a visibility/scope modifier (local).
    assert!(script(b"local x y = 5", MAX_DEPTH).is_err());

    // Creates a local variable x, set to 4, and a global variable y set to 5.
    // Not supported because it uses a visibility/scope modifier (local).
    assert!(script(b"local x = 4 y = 5", MAX_DEPTH).is_err());

    Ok(())
}
