//! String literal tests
mod common;

use crate::common::{check, should_error, MAX_DEPTH};
use serde_luaq::{lua_value, LuaTableEntry, LuaValue};

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
wasm_bindgen_test_configure!(run_in_browser);

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn basics() {
    // Empty string
    check(b"\"\"", LuaValue::String(b"".into()));
    check(b"''", LuaValue::String(b"".into()));

    check(b"\"hello world\"", LuaValue::String(b"hello world".into()));
    check(b"'hello world'", LuaValue::String(b"hello world".into()));
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn long_string() {
    check(b"[[]]", LuaValue::String(b"".into()));
    check(b"[=[]=]", LuaValue::String(b"".into()));
    check(b"[==[]==]", LuaValue::String(b"".into()));
    check(b"[===[]===]", LuaValue::String(b"".into()));
    check(b"[====[]====]", LuaValue::String(b"".into()));
    check(b"[=====[]=====]", LuaValue::String(b"".into()));

    check(b"[[hello world]]", LuaValue::String(b"hello world".into()));
    check(
        b"[=[hello world]=]",
        LuaValue::String(b"hello world".into()),
    );
    check(
        b"[==[hello world]==]",
        LuaValue::String(b"hello world".into()),
    );
    check(
        b"[===[hello world]===]",
        LuaValue::String(b"hello world".into()),
    );
    check(
        b"[====[hello world]====]",
        LuaValue::String(b"hello world".into()),
    );
    check(
        b"[=====[hello world]=====]",
        LuaValue::String(b"hello world".into()),
    );

    // Newlines
    // Lua normalises these to the platform's newline character, but we retain these as-is because
    // it could otherwise affect unescaped binary data.
    check(
        b"[=[hello \n\n world]=]",
        LuaValue::String(b"hello \n\n world".into()),
    );
    check(
        b"[=[hello \r\r world]=]",
        LuaValue::String(b"hello \r\r world".into()),
    );
    check(
        b"[=[hello \r\n world]=]",
        LuaValue::String(b"hello \r\n world".into()),
    );
    check(
        b"[=[hello \n\r world]=]",
        LuaValue::String(b"hello \n\r world".into()),
    );

    // Escape sequences should not be interpreted...
    check(
        br"[[\0\001\n\r\023\t\u{65e5}]]",
        LuaValue::String(br"\0\001\n\r\023\t\u{65e5}".into()),
    );
    check(
        br"[==[\0\001\n\r\023\t\u{65e5}]==]",
        LuaValue::String(br"\0\001\n\r\023\t\u{65e5}".into()),
    );

    // ...regardless of whether they are valid
    for c in ('\0'..='\\').chain('^'..='\u{ff}') {
        let expected = format!("\\{c}");
        let i1 = format!("[[{expected}]]");
        let i2 = format!("[=[{expected}]=]");
        check(i1.as_bytes(), LuaValue::String(expected.as_bytes().into()));
        check(i2.as_bytes(), LuaValue::String(expected.as_bytes().into()));
    }

    // Long brackets may only be ended with a bracket of the same level
    check(
        b"[=[hell[==[o]==] world]=]",
        LuaValue::String(b"hell[==[o]==] world".into()),
    );
    check(
        b"[=[hell[==[o[==[ world]=]",
        LuaValue::String(b"hell[==[o[==[ world".into()),
    );
    check(
        b"[=[hell[[o]] world]=]",
        LuaValue::String(b"hell[[o]] world".into()),
    );
    check(
        b"[[hell[=[o]=] world]]",
        LuaValue::String(b"hell[=[o]=] world".into()),
    );
    check(
        b"[[hell[=[o[==[ world]]",
        LuaValue::String(b"hell[=[o[==[ world".into()),
    );

    // Mix of short and long quotes
    check(
        b"[[hell\"o\" w'o'rld]]",
        LuaValue::String(b"hell\"o\" w'o'rld".into()),
    );
    check(
        b"\"hell[[o]] w[=[o]=]rld\"",
        LuaValue::String(b"hell[[o]] w[=[o]=]rld".into()),
    );

    // Multiple types of brackets in the same value
    check(
        b"{[[hello]],[=[world]=],'!',\"?\"}",
        LuaValue::Table(vec![
            LuaTableEntry::Value(LuaValue::String(b"hello".into())),
            LuaTableEntry::Value(LuaValue::String(b"world".into())),
            LuaTableEntry::Value(LuaValue::String(b"!".into())),
            LuaTableEntry::Value(LuaValue::String(b"?".into())),
        ]),
    );

    // When a long string is used as a table key, there must be a space before
    // the long string.
    let expected = LuaValue::Table(vec![LuaTableEntry::KeyValue(
        LuaValue::String(b"a".into()),
        LuaValue::String(b"b".into()),
    )]);
    check(b"{[ [[a]]]=[[b]]}", &expected);
    check(b"{[ [=[a]=]]=[[b]]}", &expected);
    check(b"{[ [[a]]] = [[b]]}", &expected);
    check(b"{[ [[a]] ] = [[b]]}", &expected);
    should_error(b"{[[[a]]] = [[b]]}");
    should_error(b"{[[[a]] ] = [[b]]}");
    should_error(b"{c = 3, [[[a]]] = [[b]]}");
    should_error(b"{c = 3, [[[a]] ] = [[b]]}");
    should_error(b"{['a'] = 1, [\"b\"] = 2, [[[c]]] = 3, [[=[d]=]] = 4}");
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn newlines() {
    // Short strings need to escape newlines
    should_error(b"'\nfoo'");
    should_error(b"'\rfoo'");
    should_error(b"'\r\nfoo'");
    should_error(b"'\n\rfoo'");

    should_error(b"\"\nfoo\"");
    should_error(b"\"\rfoo\"");
    should_error(b"\"\r\nfoo\"");
    should_error(b"\"\n\rfoo\"");

    // Backslash with line break.
    // Lua normalises these to the platform's newline character, but we retain these as-is because
    // it could otherwise affect unescaped binary data.
    check(
        b"\"hello\\\nworld\"",
        LuaValue::String(b"hello\nworld".into()),
    );

    check(
        b"\"hello\\\r\nworld\"",
        LuaValue::String(b"hello\r\nworld".into()),
    );

    check(
        b"\"hello\\\rworld\"",
        LuaValue::String(b"hello\rworld".into()),
    );

    check(
        b"\"hello\\\n\rworld\"",
        LuaValue::String(b"hello\n\rworld".into()),
    );

    // When the opening long bracket is immediately followed by a newline, the newline is not
    // included in the string.
    let expected = LuaValue::String(b"hello".into());
    check(b"[[\nhello]]", &expected);
    check(b"[[\rhello]]", &expected);
    check(b"[[\r\nhello]]", &expected);
    check(b"[[\n\rhello]]", &expected);

    check(b"[=[\nhello]=]", &expected);
    check(b"[=[\rhello]=]", &expected);
    check(b"[=[\r\nhello]=]", &expected);
    check(b"[=[\n\rhello]=]", &expected);

    // Any whitespace characters after the initial newline are included
    let expected = LuaValue::String(b" hello".into());
    check(b"[[\n hello]]", &expected);
    check(b"[[\r hello]]", &expected);
    check(b"[[\r\n hello]]", &expected);
    check(b"[[\n\r hello]]", &expected);

    check(b"[=[\n hello]=]", &expected);
    check(b"[=[\r hello]=]", &expected);
    check(b"[=[\r\n hello]=]", &expected);
    check(b"[=[\n\r hello]=]", &expected);

    // Only the first newline is removed.
    check(b"[[\n\nhello]]", LuaValue::String(b"\nhello".into()));
    check(b"[[\r\rhello]]", LuaValue::String(b"\rhello".into()));
    check(b"[[\r\n\r\nhello]]", LuaValue::String(b"\r\nhello".into()));
    check(b"[[\n\r\n\rhello]]", LuaValue::String(b"\n\rhello".into()));

    check(b"[=[\n\nhello]=]", LuaValue::String(b"\nhello".into()));
    check(b"[=[\r\rhello]=]", LuaValue::String(b"\rhello".into()));
    check(
        b"[=[\r\n\r\nhello]=]",
        LuaValue::String(b"\r\nhello".into()),
    );
    check(
        b"[=[\n\r\n\rhello]=]",
        LuaValue::String(b"\n\rhello".into()),
    );

    // Trailing newlines are retained.
    check(b"[[\n\nhello\n]]", LuaValue::String(b"\nhello\n".into()));
    check(b"[[\r\rhello\r]]", LuaValue::String(b"\rhello\r".into()));
    check(
        b"[[\r\n\r\nhello\r\n]]",
        LuaValue::String(b"\r\nhello\r\n".into()),
    );
    check(
        b"[[\n\r\n\rhello\n\r]]",
        LuaValue::String(b"\n\rhello\n\r".into()),
    );

    check(b"[=[\n\nhello\n]=]", LuaValue::String(b"\nhello\n".into()));
    check(b"[=[\r\rhello\r]=]", LuaValue::String(b"\rhello\r".into()));
    check(
        b"[=[\r\n\r\nhello\r\n]=]",
        LuaValue::String(b"\r\nhello\r\n".into()),
    );
    check(
        b"[=[\n\r\n\rhello\n\r]=]",
        LuaValue::String(b"\n\rhello\n\r".into()),
    );
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn escapes() {
    // https://github.com/lua/tests/blob/26eebb47b6442996d89e298b99404cbf53468c4c/strings.lua#L152
    check(
        r#""\"ílo\"\n\\""#.as_bytes(),
        LuaValue::String("\"ílo\"\n\\".as_bytes().into()),
    );

    // Null bytes are allowed in strings
    check(b"\"\0\"", LuaValue::String(b"\0".into()));

    // ...so are invalid UTF-8 sequences
    check(b"\"\xFEedMe\"", LuaValue::String(b"\xFEedMe".into()));

    // ...and arbitrary binary data
    check(
        b"\"\0\x01\0023\x05\0009\"",
        LuaValue::String(b"\0\x01\0023\x05\0009".into()),
    );

    // escaped binary data
    check(
        b"\"\\0\\1\\02\\0023\\5\\0009\"",
        LuaValue::String(b"\0\x01\x02\x023\x05\09".into()),
    );

    check(
        b"\"\0\\rx\\r\\n\xFE\\0\\00\\000\\x00\\10\\010\\255\\xFF\\u{1f4a9}\"",
        LuaValue::String(b"\0\rx\r\n\xFE\0\0\0\0\x0A\x0A\xFF\xFF\xf0\x9f\x92\xa9".into()),
    );
}

/// \z escapes
/// <https://github.com/lua/tests/blob/26eebb47b6442996d89e298b99404cbf53468c4c/literals.lua#L40>
#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn z_escapes() {
    check(b"\"\\z\"", LuaValue::String(b"".into()));
    check(br"'\z'", LuaValue::String(b"".into()));
    check(br"'\z    '", LuaValue::String(b"".into()));
    check(b"\"\\z  \n\t\x0c\x0b\n\"", LuaValue::String(b"".into()));

    check(b"\"abc\\z  \n   efg\"", LuaValue::String(b"abcefg".into()));
    check(b"\"abc\\zefg\"", LuaValue::String(b"abcefg".into()));

    check(b"\"abc\\z  \n\n\n\"", LuaValue::String(b"abc".into()));

    // Escaped whitespace characters aren't whitespace
    // > a='\z  \n  '
    // > string.byte(a, 1, #a)
    // 10      32      32
    check(br"'\z  \n  '", LuaValue::String(b"\n  ".into()));
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn single_char_escapes() {
    check(br"'\a'", LuaValue::String(b"\x07".into()));
    check(b"\"\\a\"", LuaValue::String(b"\x07".into()));

    check(br"'\b'", LuaValue::String(b"\x08".into()));
    check(b"\"\\b\"", LuaValue::String(b"\x08".into()));

    check(br"'\f'", LuaValue::String(b"\x0c".into()));
    check(b"\"\\f\"", LuaValue::String(b"\x0c".into()));

    check(br"'\n'", LuaValue::String(b"\n".into()));
    check(b"\"\\n\"", LuaValue::String(b"\n".into()));

    check(br"'\r'", LuaValue::String(b"\r".into()));
    check(b"\"\\r\"", LuaValue::String(b"\r".into()));

    check(br"'\t'", LuaValue::String(b"\t".into()));
    check(b"\"\\t\"", LuaValue::String(b"\t".into()));

    check(br"'\v'", LuaValue::String(b"\x0B".into()));
    check(b"\"\\v\"", LuaValue::String(b"\x0B".into()));

    check(br"'\\'", LuaValue::String(b"\\".into()));
    check(b"\"\\\\\"", LuaValue::String(b"\\".into()));

    check(br"'\''", LuaValue::String(b"'".into()));
    check(b"\"\\'\"", LuaValue::String(b"'".into()));

    check(b"'\\\"'", LuaValue::String(b"\"".into()));
    check(b"\"\\\"\"", LuaValue::String(b"\"".into()));

    // Quotes of the other type don't need escaping
    check(b"\"'\"", LuaValue::String(b"'".into()));
    check(b"'\"'", LuaValue::String(b"\"".into()));
}

/// When an escaped newline appears in a short string, it is preserved.
///
/// Lua converts the linefeeds to the native platform's linefeeds, eg: on a system with `\n`
/// linefeeds:
///
/// ```text
/// $ printf 'a="foo\\\nbar";print(string.byte(a,1,#a))' | lua
/// 102     111     111     10      98      97      11
/// $ printf 'a="foo\\\rbar";print(string.byte(a,1,#a))' | lua
/// 102     111     111     10      98      97      11
/// $ printf 'a="foo\\\n\\rbar";print(string.byte(a,1,#a))' | lua
/// 102     111     111     10      98      97      11
/// $ printf 'a="foo\\\r\\nbar";print(string.byte(a,1,#a))' | lua
/// 102     111     111     10      98      97      11
/// ```
///
/// But we _don't_ do that.
#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn newline_escapes() {
    check(
        b"'hello\\\nworld'",
        LuaValue::String(b"hello\nworld".into()),
    );
    check(
        b"\"hello\\\nworld\"",
        LuaValue::String(b"hello\nworld".into()),
    );

    check(
        b"'hello\\\rworld'",
        LuaValue::String(b"hello\rworld".into()),
    );
    check(
        b"\"hello\\\rworld\"",
        LuaValue::String(b"hello\rworld".into()),
    );

    check(
        b"'hello\\\n\rworld'",
        LuaValue::String(b"hello\n\rworld".into()),
    );
    check(
        b"\"hello\\\n\rworld\"",
        LuaValue::String(b"hello\n\rworld".into()),
    );

    check(
        b"'hello\\\r\nworld'",
        LuaValue::String(b"hello\r\nworld".into()),
    );
    check(
        b"\"hello\\\r\nworld\"",
        LuaValue::String(b"hello\r\nworld".into()),
    );

    // Leading whitespace on the continuing line is preserved
    check(
        b"'hello\\\n  world'",
        LuaValue::String(b"hello\n  world".into()),
    );
    check(
        b"\"hello\\\n  world\"",
        LuaValue::String(b"hello\n  world".into()),
    );

    check(
        b"'hello\\\r  world'",
        LuaValue::String(b"hello\r  world".into()),
    );
    check(
        b"\"hello\\\r  world\"",
        LuaValue::String(b"hello\r  world".into()),
    );

    check(
        b"'hello\\\n\r  world'",
        LuaValue::String(b"hello\n\r  world".into()),
    );
    check(
        b"\"hello\\\n\r  world\"",
        LuaValue::String(b"hello\n\r  world".into()),
    );

    check(
        b"'hello\\\r\n  world'",
        LuaValue::String(b"hello\r\n  world".into()),
    );
    check(
        b"\"hello\\\r\n  world\"",
        LuaValue::String(b"hello\r\n  world".into()),
    );
}

/// Decimal (`\109`) and hexadecimal (`\x6d`) escapes.
#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn decimal_x_escapes() {
    for c in 0u8..=255 {
        let expected = &[c][..];
        let expected_a = &[c, b'a'][..];
        check(
            format!("'\\{c}'").as_bytes(),
            LuaValue::String(expected.into()),
        );
        check(
            format!("\"\\{c}\"").as_bytes(),
            LuaValue::String(expected.into()),
        );

        // decimal escape with letter after
        check(
            format!("'\\{c}a'").as_bytes(),
            LuaValue::String(expected_a.into()),
        );
        check(
            format!("\"\\{c}a\"").as_bytes(),
            LuaValue::String(expected_a.into()),
        );

        // \x lowercase
        check(
            format!("'\\x{c:02x}'").as_bytes(),
            LuaValue::String(expected.into()),
        );
        check(
            format!("\"\\x{c:02x}\"").as_bytes(),
            LuaValue::String(expected.into()),
        );

        // \x uppercase
        check(
            format!("'\\x{c:02X}'").as_bytes(),
            LuaValue::String(expected.into()),
        );
        check(
            format!("\"\\x{c:02X}\"").as_bytes(),
            LuaValue::String(expected.into()),
        );

        // \x with letter after
        check(
            format!("'\\x{c:02x}a'").as_bytes(),
            LuaValue::String(expected_a.into()),
        );
        check(
            format!("\"\\x{c:02x}a\"").as_bytes(),
            LuaValue::String(expected_a.into()),
        );

        check(
            format!("'\\x{c:02X}a'").as_bytes(),
            LuaValue::String(expected_a.into()),
        );
        check(
            format!("\"\\x{c:02X}a\"").as_bytes(),
            LuaValue::String(expected_a.into()),
        );

        if c < 100 {
            // With one leading zero
            check(
                format!("'\\0{c}'").as_bytes(),
                LuaValue::String(expected.into()),
            );
            check(
                format!("\"\\0{c}\"").as_bytes(),
                LuaValue::String(expected.into()),
            );

            check(
                format!("'\\0{c}a'").as_bytes(),
                LuaValue::String(expected_a.into()),
            );
            check(
                format!("\"\\0{c}a\"").as_bytes(),
                LuaValue::String(expected_a.into()),
            );
        }

        if c < 10 {
            // With two leading zeros
            check(
                format!("'\\00{c}'").as_bytes(),
                LuaValue::String(expected.into()),
            );
            check(
                format!("\"\\00{c}\"").as_bytes(),
                LuaValue::String(expected.into()),
            );

            check(
                format!("'\\00{c}a'").as_bytes(),
                LuaValue::String(expected_a.into()),
            );
            check(
                format!("\"\\00{c}a\"").as_bytes(),
                LuaValue::String(expected_a.into()),
            );

            // We should allow a digit after for three-digit escapes
            let expected_1 = &[c, b'1'][..];
            check(
                format!("'\\00{c}1'").as_bytes(),
                LuaValue::String(expected_1.into()),
            );
            check(
                format!("\"\\00{c}1\"").as_bytes(),
                LuaValue::String(expected_1.into()),
            );
        }
    }

    // Invalid decimal escapes
    for c in 256..=999 {
        should_error(format!("'\\{c}'").as_bytes());
        should_error(format!("\"\\{c}\"").as_bytes());
    }
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn unicode_escapes() {
    check(br"'\u{0}'", LuaValue::String(b"\0".into()));
    check(br"'\u{00}'", LuaValue::String(b"\0".into()));
    check(br"'\u{00000000}'", LuaValue::String(b"\0".into()));
    check(br"'\u{0000000000000000}'", LuaValue::String(b"\0".into()));
    check(br"'\u{80}'", LuaValue::String(b"\xC2\x80".into()));
    check(
        br"'\u{10FFFF}'",
        LuaValue::String(b"\xF4\x8F\xBF\xBF".into()),
    );
    check(
        br"'\u{0000000000000010FFFF}'",
        LuaValue::String(b"\xF4\x8F\xBF\xBF".into()),
    );

    check(
        br"'\u{65E5}\u{672c}\u{8A9e}'",
        LuaValue::String("\u{65E5}\u{672C}\u{8a9E}".as_bytes().into()),
    );

    // Lua follows RFC 2279 UTF-8 (1998), so allows some sequences that were
    // later *disallowed* in RFC 3629 UTF-8 (2003).
    check(br"'\u{d800}'", LuaValue::String(b"\xED\xA0\x80".into()));
    check(br"'\u{dfff}'", LuaValue::String(b"\xED\xBF\xBF".into()));
    check(
        br"'\u{110000}'",
        LuaValue::String(b"\xF4\x90\x80\x80".into()),
    );

    check(
        br"'\u{70000000}'",
        LuaValue::String(b"\xFD\xB0\x80\x80\x80\x80".into()),
    );
    check(
        br"'\u{7fffffff}'",
        LuaValue::String(b"\xFD\xBF\xBF\xBF\xBF\xBF".into()),
    );
    check(
        br"'\u{07fffffff}'",
        LuaValue::String(b"\xFD\xBF\xBF\xBF\xBF\xBF".into()),
    );
    check(
        br"'\u{00000000000000007fffffff}'",
        LuaValue::String(b"\xFD\xBF\xBF\xBF\xBF\xBF".into()),
    );

    // Strings should not be normalised
    check(
        b"'fran\xC3\xA7ais'",
        LuaValue::String(b"fran\xc3\xa7ais".into()),
    );
    check(
        br"'fran\xC3\xA7ais'",
        LuaValue::String(b"fran\xc3\xa7ais".into()),
    );
    check(
        br"'fran\u{E7}ais'",
        LuaValue::String(b"fran\xc3\xa7ais".into()),
    );

    check(
        b"'franc\xCC\xA7ais'",
        LuaValue::String(b"franc\xcc\xa7ais".into()),
    );
    check(
        br"'franc\xCC\xA7ais'",
        LuaValue::String(b"franc\xcc\xa7ais".into()),
    );
    check(
        br"'franc\u{327}ais'",
        LuaValue::String(b"franc\xcc\xa7ais".into()),
    );

    // Invalid
    should_error(br"'\u'");
    should_error(br"'\u '");
    should_error(br"'\u\n'");
    should_error(br"'\u12'");
    should_error(br"'\uab'");
    should_error(br"'\u{'");
    should_error(br"'\u{}'");
    should_error(br"'\u{ }'");
    should_error(br"'\u{hello}'");
    should_error(br"'\u{80000000}'");
    should_error(br"'\u{-80}'");
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn invalid_escapes() {
    should_error(br"'\ '");
    should_error(b"'\\\t'");

    for l in ('\0'..='\x09'/* \t */)
        // \n
        .chain(['\x0b', '\x0c'])
        // \r
        .chain('\x0e'..='!')
        // double quote
        .chain('#'..='&')
        // single quote
        .chain('('..='/')
        // digits
        .chain(/* uppercase letters */ ':'..='[')
        // backslash
        .chain(']'..='`')
        // a, b
        .chain('c'..='e')
        // f
        .chain('g'..='m')
        // n
        .chain('o'..='q')
        // r, t, v, z
        .chain([
            's', /* ... */
            'u', /* u alone is invalid */
            'w', /* ... */
            'x', /* x alone is invalid */
            'y', /* ... */
        ])
        .chain('{'..='\u{ff}')
    {
        should_error(format!("'\\{l}'").as_bytes());
        should_error(format!("\"\\{l}\"").as_bytes());
    }

    // x escapes need two hex digits, not just one
    for x in 0..=0xf {
        should_error(format!("'\\x{x:x}'").as_bytes());
        should_error(format!("\"\\x{x:x}\"").as_bytes());
    }

    // x escapes don't accept other characters after
    should_error(br"'\xyz'");

    // Incomplete strings
    should_error(br"'\'");
    should_error(b"\"\\\"");
}

#[test]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
fn borrows() -> Result {
    // Empty strings
    assert!(lua_value(b"[[]]", MAX_DEPTH)?.is_borrowed());
    assert!(lua_value(b"[=[]=]", MAX_DEPTH)?.is_borrowed());
    assert!(lua_value(b"''", MAX_DEPTH)?.is_borrowed());
    assert!(lua_value(b"\"\"", MAX_DEPTH)?.is_borrowed());

    // No escape sequences
    assert!(lua_value(b"[[hello]]", MAX_DEPTH)?.is_borrowed());
    assert!(lua_value(b"[=[hello]=]", MAX_DEPTH)?.is_borrowed());
    assert!(lua_value(b"'hello'", MAX_DEPTH)?.is_borrowed());
    assert!(lua_value(b"\"hello\"", MAX_DEPTH)?.is_borrowed());

    // Escapes are ignored for long bracket strings, so should be borrowed
    assert!(lua_value(br"[[hello\nworld]]", MAX_DEPTH)?.is_borrowed());
    assert!(lua_value(br"[=[hello\nworld]=]", MAX_DEPTH)?.is_borrowed());

    // Newline character should also be included
    assert!(lua_value(b"[[hello\nworld]]", MAX_DEPTH)?.is_borrowed());
    assert!(lua_value(b"[=[hello\nworld]=]", MAX_DEPTH)?.is_borrowed());

    // Strings containing _only_ an escape are borrowed
    assert!(lua_value(br"'\n'", MAX_DEPTH)?.is_borrowed());
    assert!(lua_value(b"\"\\n\"", MAX_DEPTH)?.is_borrowed());

    // Strings containing multiple escapes are owned
    assert!(!lua_value(br"'\r\n'", MAX_DEPTH)?.is_borrowed());
    assert!(!lua_value(b"\"\\r\\n\"", MAX_DEPTH)?.is_borrowed());

    // Strings containing escapes and non-escaped are owned
    assert!(!lua_value(br"'hello\n'", MAX_DEPTH)?.is_borrowed());
    assert!(!lua_value(b"\"hello\\n\"", MAX_DEPTH)?.is_borrowed());

    Ok(())
}
