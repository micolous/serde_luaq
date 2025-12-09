mod common;
use crate::common::MAX_DEPTH;

use serde_luaq::{lua_value, return_statement, script, LuaTableEntry, LuaValue};

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
wasm_bindgen_test_configure!(run_in_browser);

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn keywords() -> Result {
    // Assignment to keyword is invalid.
    assert!(script(b"and = true\n", MAX_DEPTH).is_err());
    assert!(script(b"break = true\n", MAX_DEPTH).is_err());
    assert!(script(b"do = true\n", MAX_DEPTH).is_err());
    assert!(script(b"else = true\n", MAX_DEPTH).is_err());
    assert!(script(b"elseif = true\n", MAX_DEPTH).is_err());
    assert!(script(b"end = true\n", MAX_DEPTH).is_err());
    assert!(script(b"false = true\n", MAX_DEPTH).is_err());
    assert!(script(b"for = true\n", MAX_DEPTH).is_err());
    assert!(script(b"function = true\n", MAX_DEPTH).is_err());
    assert!(script(b"goto = true\n", MAX_DEPTH).is_err());
    assert!(script(b"if = true\n", MAX_DEPTH).is_err());
    assert!(script(b"in = true\n", MAX_DEPTH).is_err());
    assert!(script(b"local = true\n", MAX_DEPTH).is_err());
    assert!(script(b"nil = true\n", MAX_DEPTH).is_err());
    assert!(script(b"not = true\n", MAX_DEPTH).is_err());
    assert!(script(b"or = true\n", MAX_DEPTH).is_err());
    assert!(script(b"repeat = true\n", MAX_DEPTH).is_err());
    assert!(script(b"return = true\n", MAX_DEPTH).is_err());
    assert!(script(b"then = true\n", MAX_DEPTH).is_err());
    assert!(script(b"true = true\n", MAX_DEPTH).is_err());
    assert!(script(b"until = true\n", MAX_DEPTH).is_err());
    assert!(script(b"while = true\n", MAX_DEPTH).is_err());

    // Keywords used as identifier in table keys is invalid.
    assert!(return_statement(b"return {and = true}", MAX_DEPTH).is_err());
    assert!(return_statement(b"return {break = true}", MAX_DEPTH).is_err());
    assert!(return_statement(b"return {do = true}", MAX_DEPTH).is_err());
    assert!(return_statement(b"return {else = true}", MAX_DEPTH).is_err());
    assert!(return_statement(b"return {elseif = true}", MAX_DEPTH).is_err());
    assert!(return_statement(b"return {end = true}", MAX_DEPTH).is_err());
    assert!(return_statement(b"return {false = true}", MAX_DEPTH).is_err());
    assert!(return_statement(b"return {for = true}", MAX_DEPTH).is_err());
    assert!(return_statement(b"return {function = true}", MAX_DEPTH).is_err());
    assert!(return_statement(b"return {goto = true}", MAX_DEPTH).is_err());
    assert!(return_statement(b"return {if = true}", MAX_DEPTH).is_err());
    assert!(return_statement(b"return {in = true}", MAX_DEPTH).is_err());
    assert!(return_statement(b"return {local = true}", MAX_DEPTH).is_err());
    assert!(return_statement(b"return {nil = true}", MAX_DEPTH).is_err());
    assert!(return_statement(b"return {not = true}", MAX_DEPTH).is_err());
    assert!(return_statement(b"return {or = true}", MAX_DEPTH).is_err());
    assert!(return_statement(b"return {repeat = true}", MAX_DEPTH).is_err());
    assert!(return_statement(b"return {return = true}", MAX_DEPTH).is_err());
    assert!(return_statement(b"return {then = true}", MAX_DEPTH).is_err());
    assert!(return_statement(b"return {true = true}", MAX_DEPTH).is_err());
    assert!(return_statement(b"return {until = true}", MAX_DEPTH).is_err());
    assert!(return_statement(b"return {while = true}", MAX_DEPTH).is_err());

    assert!(lua_value(b"{and = true}", MAX_DEPTH).is_err());
    assert!(lua_value(b"{break = true}", MAX_DEPTH).is_err());
    assert!(lua_value(b"{do = true}", MAX_DEPTH).is_err());
    assert!(lua_value(b"{else = true}", MAX_DEPTH).is_err());
    assert!(lua_value(b"{elseif = true}", MAX_DEPTH).is_err());
    assert!(lua_value(b"{end = true}", MAX_DEPTH).is_err());
    assert!(lua_value(b"{false = true}", MAX_DEPTH).is_err());
    assert!(lua_value(b"{for = true}", MAX_DEPTH).is_err());
    assert!(lua_value(b"{function = true}", MAX_DEPTH).is_err());
    assert!(lua_value(b"{goto = true}", MAX_DEPTH).is_err());
    assert!(lua_value(b"{if = true}", MAX_DEPTH).is_err());
    assert!(lua_value(b"{in = true}", MAX_DEPTH).is_err());
    assert!(lua_value(b"{local = true}", MAX_DEPTH).is_err());
    assert!(lua_value(b"{nil = true}", MAX_DEPTH).is_err());
    assert!(lua_value(b"{not = true}", MAX_DEPTH).is_err());
    assert!(lua_value(b"{or = true}", MAX_DEPTH).is_err());
    assert!(lua_value(b"{repeat = true}", MAX_DEPTH).is_err());
    assert!(lua_value(b"{return = true}", MAX_DEPTH).is_err());
    assert!(lua_value(b"{then = true}", MAX_DEPTH).is_err());
    assert!(lua_value(b"{true = true}", MAX_DEPTH).is_err());
    assert!(lua_value(b"{until = true}", MAX_DEPTH).is_err());
    assert!(lua_value(b"{while = true}", MAX_DEPTH).is_err());

    // Keywords used as table key in strings should be accepted.
    assert_eq!(
        LuaValue::Table(vec![LuaTableEntry::KeyValue(
            b"and".into(),
            LuaValue::Boolean(true)
        ),]),
        lua_value(b"{[\"and\"] = true}", MAX_DEPTH)?,
    );
    assert_eq!(
        LuaValue::Table(vec![LuaTableEntry::KeyValue(
            b"break".into(),
            LuaValue::Boolean(true)
        ),]),
        lua_value(b"{[\"break\"] = true}", MAX_DEPTH)?,
    );
    assert_eq!(
        LuaValue::Table(vec![LuaTableEntry::KeyValue(
            b"do".into(),
            LuaValue::Boolean(true)
        ),]),
        lua_value(b"{[\"do\"] = true}", MAX_DEPTH)?,
    );
    assert_eq!(
        LuaValue::Table(vec![LuaTableEntry::KeyValue(
            b"else".into(),
            LuaValue::Boolean(true)
        ),]),
        lua_value(b"{[\"else\"] = true}", MAX_DEPTH)?,
    );
    assert_eq!(
        LuaValue::Table(vec![LuaTableEntry::KeyValue(
            b"elseif".into(),
            LuaValue::Boolean(true)
        ),]),
        lua_value(b"{[\"elseif\"] = true}", MAX_DEPTH)?,
    );
    assert_eq!(
        LuaValue::Table(vec![LuaTableEntry::KeyValue(
            b"end".into(),
            LuaValue::Boolean(true)
        ),]),
        lua_value(b"{[\"end\"] = true}", MAX_DEPTH)?,
    );
    assert_eq!(
        LuaValue::Table(vec![LuaTableEntry::KeyValue(
            b"false".into(),
            LuaValue::Boolean(true)
        ),]),
        lua_value(b"{[\"false\"] = true}", MAX_DEPTH)?,
    );
    assert_eq!(
        LuaValue::Table(vec![LuaTableEntry::KeyValue(
            b"for".into(),
            LuaValue::Boolean(true)
        ),]),
        lua_value(b"{[\"for\"] = true}", MAX_DEPTH)?,
    );
    assert_eq!(
        LuaValue::Table(vec![LuaTableEntry::KeyValue(
            b"function".into(),
            LuaValue::Boolean(true)
        ),]),
        lua_value(b"{[\"function\"] = true}", MAX_DEPTH)?,
    );
    assert_eq!(
        LuaValue::Table(vec![LuaTableEntry::KeyValue(
            b"goto".into(),
            LuaValue::Boolean(true)
        ),]),
        lua_value(b"{[\"goto\"] = true}", MAX_DEPTH)?,
    );
    assert_eq!(
        LuaValue::Table(vec![LuaTableEntry::KeyValue(
            b"if".into(),
            LuaValue::Boolean(true)
        ),]),
        lua_value(b"{[\"if\"] = true}", MAX_DEPTH)?,
    );
    assert_eq!(
        LuaValue::Table(vec![LuaTableEntry::KeyValue(
            b"in".into(),
            LuaValue::Boolean(true)
        ),]),
        lua_value(b"{[\"in\"] = true}", MAX_DEPTH)?,
    );
    assert_eq!(
        LuaValue::Table(vec![LuaTableEntry::KeyValue(
            b"local".into(),
            LuaValue::Boolean(true)
        ),]),
        lua_value(b"{[\"local\"] = true}", MAX_DEPTH)?,
    );
    assert_eq!(
        LuaValue::Table(vec![LuaTableEntry::KeyValue(
            b"nil".into(),
            LuaValue::Boolean(true)
        ),]),
        lua_value(b"{[\"nil\"] = true}", MAX_DEPTH)?,
    );
    assert_eq!(
        LuaValue::Table(vec![LuaTableEntry::KeyValue(
            b"not".into(),
            LuaValue::Boolean(true)
        ),]),
        lua_value(b"{[\"not\"] = true}", MAX_DEPTH)?,
    );
    assert_eq!(
        LuaValue::Table(vec![LuaTableEntry::KeyValue(
            b"or".into(),
            LuaValue::Boolean(true)
        ),]),
        lua_value(b"{[\"or\"] = true}", MAX_DEPTH)?,
    );
    assert_eq!(
        LuaValue::Table(vec![LuaTableEntry::KeyValue(
            b"repeat".into(),
            LuaValue::Boolean(true)
        ),]),
        lua_value(b"{[\"repeat\"] = true}", MAX_DEPTH)?,
    );
    assert_eq!(
        LuaValue::Table(vec![LuaTableEntry::KeyValue(
            b"return".into(),
            LuaValue::Boolean(true)
        ),]),
        lua_value(b"{[\"return\"] = true}", MAX_DEPTH)?,
    );
    assert_eq!(
        LuaValue::Table(vec![LuaTableEntry::KeyValue(
            b"then".into(),
            LuaValue::Boolean(true)
        ),]),
        lua_value(b"{[\"then\"] = true}", MAX_DEPTH)?,
    );
    assert_eq!(
        LuaValue::Table(vec![LuaTableEntry::KeyValue(
            b"true".into(),
            LuaValue::Boolean(true)
        ),]),
        lua_value(b"{[\"true\"] = true}", MAX_DEPTH)?,
    );
    assert_eq!(
        LuaValue::Table(vec![LuaTableEntry::KeyValue(
            b"until".into(),
            LuaValue::Boolean(true)
        ),]),
        lua_value(b"{[\"until\"] = true}", MAX_DEPTH)?,
    );
    assert_eq!(
        LuaValue::Table(vec![LuaTableEntry::KeyValue(
            b"while".into(),
            LuaValue::Boolean(true)
        ),]),
        lua_value(b"{[\"while\"] = true}", MAX_DEPTH)?,
    );

    Ok(())
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn contains_keyword() -> Result {
    // Starts with a keyword
    assert_eq!(
        vec![("and1", LuaValue::Boolean(true)),],
        script(b"and1 = true\n", MAX_DEPTH)?
    );
    assert_eq!(
        vec![("break1", LuaValue::Boolean(true)),],
        script(b"break1 = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("do1", LuaValue::Boolean(true)),],
        script(b"do1 = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("else1", LuaValue::Boolean(true)),],
        script(b"else1 = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("elseif1", LuaValue::Boolean(true)),],
        script(b"elseif1 = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("end1", LuaValue::Boolean(true)),],
        script(b"end1 = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("false1", LuaValue::Boolean(true)),],
        script(b"false1 = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("for1", LuaValue::Boolean(true)),],
        script(b"for1 = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("function1", LuaValue::Boolean(true)),],
        script(b"function1 = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("goto1", LuaValue::Boolean(true)),],
        script(b"goto1 = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("if1", LuaValue::Boolean(true)),],
        script(b"if1 = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("in1", LuaValue::Boolean(true)),],
        script(b"in1 = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("local1", LuaValue::Boolean(true)),],
        script(b"local1 = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("nil1", LuaValue::Boolean(true)),],
        script(b"nil1 = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("not1", LuaValue::Boolean(true)),],
        script(b"not1 = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("or1", LuaValue::Boolean(true)),],
        script(b"or1 = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("repeat1", LuaValue::Boolean(true)),],
        script(b"repeat1 = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("return1", LuaValue::Boolean(true)),],
        script(b"return1 = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("then1", LuaValue::Boolean(true)),],
        script(b"then1 = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("true1", LuaValue::Boolean(true)),],
        script(b"true1 = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("until1", LuaValue::Boolean(true)),],
        script(b"until1 = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("while1", LuaValue::Boolean(true)),],
        script(b"while1 = true\n", MAX_DEPTH)?,
    );

    // Ends with a keyword
    assert_eq!(
        vec![("_and", LuaValue::Boolean(true)),],
        script(b"_and = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("_break", LuaValue::Boolean(true)),],
        script(b"_break = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("_do", LuaValue::Boolean(true)),],
        script(b"_do = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("_else", LuaValue::Boolean(true)),],
        script(b"_else = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("_elseif", LuaValue::Boolean(true)),],
        script(b"_elseif = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("_end", LuaValue::Boolean(true)),],
        script(b"_end = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("_false", LuaValue::Boolean(true)),],
        script(b"_false = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("_for", LuaValue::Boolean(true)),],
        script(b"_for = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("_function", LuaValue::Boolean(true)),],
        script(b"_function = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("_goto", LuaValue::Boolean(true)),],
        script(b"_goto = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("_if", LuaValue::Boolean(true)),],
        script(b"_if = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("_in", LuaValue::Boolean(true)),],
        script(b"_in = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("_local", LuaValue::Boolean(true)),],
        script(b"_local = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("_nil", LuaValue::Boolean(true)),],
        script(b"_nil = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("_not", LuaValue::Boolean(true)),],
        script(b"_not = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("_or", LuaValue::Boolean(true)),],
        script(b"_or = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("_repeat", LuaValue::Boolean(true)),],
        script(b"_repeat = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("_return", LuaValue::Boolean(true)),],
        script(b"_return = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("_then", LuaValue::Boolean(true)),],
        script(b"_then = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("_true", LuaValue::Boolean(true)),],
        script(b"_true = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("_until", LuaValue::Boolean(true)),],
        script(b"_until = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("_while", LuaValue::Boolean(true)),],
        script(b"_while = true\n", MAX_DEPTH)?,
    );

    // Keyword not in lower case
    assert_eq!(
        vec![("AND", LuaValue::Boolean(true)),],
        script(b"AND = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("BREAK", LuaValue::Boolean(true)),],
        script(b"BREAK = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("DO", LuaValue::Boolean(true)),],
        script(b"DO = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("ELSE", LuaValue::Boolean(true)),],
        script(b"ELSE = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("ELSEIF", LuaValue::Boolean(true)),],
        script(b"ELSEIF = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("END", LuaValue::Boolean(true)),],
        script(b"END = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("FALSE", LuaValue::Boolean(true)),],
        script(b"FALSE = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("FOR", LuaValue::Boolean(true)),],
        script(b"FOR = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("FUNCTION", LuaValue::Boolean(true)),],
        script(b"FUNCTION = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("GOTO", LuaValue::Boolean(true)),],
        script(b"GOTO = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("IF", LuaValue::Boolean(true)),],
        script(b"IF = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("IN", LuaValue::Boolean(true)),],
        script(b"IN = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("LOCAL", LuaValue::Boolean(true)),],
        script(b"LOCAL = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("NIL", LuaValue::Boolean(true)),],
        script(b"NIL = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("NOT", LuaValue::Boolean(true)),],
        script(b"NOT = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("OR", LuaValue::Boolean(true)),],
        script(b"OR = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("REPEAT", LuaValue::Boolean(true)),],
        script(b"REPEAT = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("RETURN", LuaValue::Boolean(true)),],
        script(b"RETURN = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("THEN", LuaValue::Boolean(true)),],
        script(b"THEN = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("TRUE", LuaValue::Boolean(true)),],
        script(b"TRUE = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("UNTIL", LuaValue::Boolean(true)),],
        script(b"UNTIL = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("WHILE", LuaValue::Boolean(true)),],
        script(b"WHILE = true\n", MAX_DEPTH)?,
    );

    assert_eq!(
        vec![("And", LuaValue::Boolean(true)),],
        script(b"And = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("Break", LuaValue::Boolean(true)),],
        script(b"Break = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("Do", LuaValue::Boolean(true)),],
        script(b"Do = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("Else", LuaValue::Boolean(true)),],
        script(b"Else = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("Elseif", LuaValue::Boolean(true)),],
        script(b"Elseif = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("End", LuaValue::Boolean(true)),],
        script(b"End = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("False", LuaValue::Boolean(true)),],
        script(b"False = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("For", LuaValue::Boolean(true)),],
        script(b"For = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("Function", LuaValue::Boolean(true)),],
        script(b"Function = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("Goto", LuaValue::Boolean(true)),],
        script(b"Goto = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("If", LuaValue::Boolean(true)),],
        script(b"If = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("In", LuaValue::Boolean(true)),],
        script(b"In = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("Local", LuaValue::Boolean(true)),],
        script(b"Local = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("Nil", LuaValue::Boolean(true)),],
        script(b"Nil = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("Not", LuaValue::Boolean(true)),],
        script(b"Not = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("Or", LuaValue::Boolean(true)),],
        script(b"Or = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("Repeat", LuaValue::Boolean(true)),],
        script(b"Repeat = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("Return", LuaValue::Boolean(true)),],
        script(b"Return = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("Then", LuaValue::Boolean(true)),],
        script(b"Then = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("True", LuaValue::Boolean(true)),],
        script(b"True = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("Until", LuaValue::Boolean(true)),],
        script(b"Until = true\n", MAX_DEPTH)?,
    );
    assert_eq!(
        vec![("While", LuaValue::Boolean(true)),],
        script(b"While = true\n", MAX_DEPTH)?,
    );

    Ok(())
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn invalid() {
    // Starts with a number
    assert!(script(b"1a = true\n", MAX_DEPTH).is_err());
    assert!(script(b"2a = true\n", MAX_DEPTH).is_err());
    assert!(script(b"3a = true\n", MAX_DEPTH).is_err());
    assert!(script(b"4a = true\n", MAX_DEPTH).is_err());
    assert!(script(b"5a = true\n", MAX_DEPTH).is_err());
    assert!(script(b"6a = true\n", MAX_DEPTH).is_err());
    assert!(script(b"7a = true\n", MAX_DEPTH).is_err());
    assert!(script(b"8a = true\n", MAX_DEPTH).is_err());
    assert!(script(b"9a = true\n", MAX_DEPTH).is_err());
    assert!(script(b"0a = true\n", MAX_DEPTH).is_err());

    // Is a number
    assert!(script(b"1 = true\n", MAX_DEPTH).is_err());
    assert!(script(b"2 = true\n", MAX_DEPTH).is_err());
    assert!(script(b"3 = true\n", MAX_DEPTH).is_err());
    assert!(script(b"4 = true\n", MAX_DEPTH).is_err());
    assert!(script(b"5 = true\n", MAX_DEPTH).is_err());
    assert!(script(b"6 = true\n", MAX_DEPTH).is_err());
    assert!(script(b"7 = true\n", MAX_DEPTH).is_err());
    assert!(script(b"8 = true\n", MAX_DEPTH).is_err());
    assert!(script(b"9 = true\n", MAX_DEPTH).is_err());
    assert!(script(b"0 = true\n", MAX_DEPTH).is_err());

    // Empty
    assert!(script(b" = true\n", MAX_DEPTH).is_err());

    // Other characters
    assert!(script(b"[] = true\n", MAX_DEPTH).is_err());
    assert!(script(b"[[]] = true\n", MAX_DEPTH).is_err());
    assert!(script(b"{} = true\n", MAX_DEPTH).is_err());
    assert!(script(b"$ = true\n", MAX_DEPTH).is_err());
    assert!(script(b"\"\" = true\n", MAX_DEPTH).is_err());
    assert!(script(b"'' = true\n", MAX_DEPTH).is_err());
    assert!(script(b"\\ = true\n", MAX_DEPTH).is_err());
    assert!(script("Français = true\n".as_bytes(), MAX_DEPTH).is_err());

    // Inside table identifiers
    assert!(script("{Français = true}\n".as_bytes(), MAX_DEPTH).is_err());
    assert!(script(b"{1 = true}\n", MAX_DEPTH).is_err());
    assert!(script(b"{[[foo]] = true}\n", MAX_DEPTH).is_err());
    assert!(script(b"{'foo' = true}\n", MAX_DEPTH).is_err());
    assert!(script(b"{\"foo\" = true}\n", MAX_DEPTH).is_err());
}
