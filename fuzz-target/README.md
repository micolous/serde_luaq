# serde_luaq fuzz target

Experiments with a really simple binary target for using `serde_luaq` with [afl.rs][], a Rust fuzzer
for [AFLplusplus][].

## Building

[Setup `cargo-afl`][afl-setup], then build with:

```sh
cargo afl build
```

You can also do a release build by adding the `--release` flag.

## Running

```sh
cargo afl fuzz -i in -o /tmp/out ../target/debug/fuzz-target
```

Replace `debug` with `release` if you did a release build.

## Inputs

This target expects treats all inputs to be Lua scripts, eg:

```lua
hello = "world"
```

The maximum table depth is set to 200, and there is no data size limit.
**Using `serde_luaq` this way is a bad idea.**

There are some pre-canned inputs in the [`in` directory](./in/):

- `braces_8k.lua`: a 8192-level nested table, which would trigger a stack overflow if not for recursion depth limits; would be rejected by Lua
- `dental_plan.lua`: a 119-level nested table, which Lua 5.4 handles just fine
- `hello.lua`: hello world!
- `integers_dec.lua`, `integers_hex.lua`: integer representations, from the main test suite
- `logo.lua`: wrapping a PNG file in a little Lua
- `numbers.lua`: various integer representations

[afl.rs]: https://github.com/rust-fuzz/afl.rs
[afl-setup]: https://rust-fuzz.github.io/book/afl/setup.html
[AFLplusplus]: https://aflplus.plus/
