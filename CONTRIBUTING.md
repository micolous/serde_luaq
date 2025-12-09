# Contributing to `serde_luaq`

Thank you for your interest in contributing to this project!

## Scope

### Lua

This library targets compatibility with Lua 5.4 (the current stable version, at the time of writing)
and data structures that have JSON equivalents (eg: `table` is like `Array` or `Object`).

Compatibility with older versions of Lua may be considered if compelling and widespread enough, and
that they don't break compatibility with Lua 5.4. Aside from identifier restrictions, the integer
subtype and float coercion changes (in Lua 5.2, 5.3 and 5.4 respectively), I don't expect this to be
a big issue.

Executing arbitrary code and Luau language features are explicitly out of scope.
_Try [`mlua`][mlua] instead!_

### Rust

This project targets the current stable version of Rust on
[64-bit platforms with Tier 1 with Host Tools][rust-tier] and [WASM][rust-wasm].

Support for stable versions up to 1 year old is on a "best effort" basis, and support for other
platforms is on a "if you do the work, it's simple and testable in CI" basis.

### Parser

This library is built around [a PEG parser][peg-parser] to simplify things, so needs to be able to
keep your entire Lua script in memory. It will avoid making copies of that script where possible,
but there are cases where this is unavoidable.

Ideally this should result in memory usage that is _not significantly worse_ than evaluating the
source code with Lua... but this is not currently measured.

## Submitting issues

- Issues must be written in English.

- For [security issues, check here][security].

- If you've found a bug, include a simple way to reproduce it. I don't have access to your code
  base, or your computer.

- [Be mindful of the scope](#scope).

- Don't report that a dependency is out of date. _Write a PR instead!_

- "How do I use Serde" questions [should see here][serde-help].

- If you have a question about how something in _this_ repository works, please use
  [the discussions tab][discussions].

  The goal is that these will be answered by the documentation.

Issues and questions sent by email will attract my professional consulting rates. :)

## Submitting PRs

- _Write an issue report first for any non-trivial change._ This will help find if someone else is
  working on the issue, or if the work is even worth persuing (ie: in scope).
- PR descriptions must be written in English.
- All contributions **must be your own work**.
- Automated PRs and the use of generative AI **is not welcome here**.
- Keep your PR small and simple. Focus on one issue at a time.
- Include test cases.
- Avoid mixing whitespace and formatting changes with your change.
- If you're fixing a security bug, [use the private vulnerability reporting system][security] to
  make a private PR.

Examples of welcomed contributions:

- Fixing Serde issues
- Expanding test cases
- Fixing [security issues][security]
- Fixing Lua parser issues
- Performance improvements
- Fixing documentation issues
- Updating outdated dependencies

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
this project by you, as defined in the Apache-2.0 license, shall be dual licensed under the terms of
the Apache-2.0 and MIT licenses, without any additional terms or conditions. 

[discussions]: https://github.com/micolous/serde_luaq/discussions
[mlua]: https://github.com/mlua-rs/mlua
[peg-parser]: https://en.wikipedia.org/wiki/Parsing_expression_grammar
[serde-help]: https://serde.rs/help.html
[rust-tier]: https://doc.rust-lang.org/nightly/rustc/platform-support.html
[rust-wasm]: https://doc.rust-lang.org/nightly/rustc/platform-support/wasm32-unknown-unknown.html
[security]: https://github.com/micolous/serde_luaq/security
