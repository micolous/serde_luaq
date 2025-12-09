mod common;
use serde_luaq::LuaValue;

use crate::common::check;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
wasm_bindgen_test_configure!(run_in_browser);

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
