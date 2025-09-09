//! Serde deserialiser example.
//!
//! Based on [`mlua`'s `serde` example][0].
//!
//! [0]: https://github.com/mlua-rs/mlua/blob/main/examples/serde.rs

use serde::{Deserialize, Serialize};
use serde_luaq::{from_slice, lua_value, LuaFormat, LuaValue};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Serialize, Deserialize)]
enum Transmission {
    Manual,
    Automatic,
}

#[derive(Debug, Serialize, Deserialize)]
struct Engine {
    v: u32,
    kw: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Car {
    active: bool,
    model: String,
    transmission: Transmission,
    engine: Engine,
}

fn main() -> Result<()> {
    let car: Car = from_slice(br#"
        {active = true, model = "Volkswagen Golf", transmission = "Automatic", engine = {v = 1499, kw = 90}}
    "#, LuaFormat::Value)?;

    println!("Car: {car:?}");

    // serde_luaq doesn't support all the things mlua does, because there's no Lua engine behind it.
    // However, serde_luaq allows setting table values to `nil`, without requiring a `null` pointer.
    let val: LuaValue = lua_value(
        br#"
        {driver = "Boris", price = nil}
    "#,
    )?;

    println!("Val: {val:?}");

    Ok(())
}
