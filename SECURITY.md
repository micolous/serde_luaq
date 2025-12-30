# Security policy

Thank you for your interest in improving the security of this project!

## Supported versions

Only [the latest stable release of the library][latest] is supported.

## Scope

Issues must be reproducible with the binary built in release mode with appropriate CPU optimisations
for a reasonably modern system with a reasonable amount of available RAM for the Lua input.

Enforcement of reasonable input size limits and setting a reasonable maximum table depth limit is
the responsibility of the _caller_. See [the latest `main` branch docs][main-mem] or
[stable docs][stable-mem] for more details.

`serde_luaq` always deserialises the entire input, and some Lua language constructs come with
significant memory overheads. It's possible for a 64 MiB file to take several seconds to load, and
require 3 GiB of RAM... [and that's fine][main-mem].

In a release build, `serde_luaq` should be able to deserialise arbitrary inputs of upto 64 MiB on a
reasonably-specced PC, in a way that _is not significantly worse_ than evaluating the input with
the current stable version of Lua (using `require()`, `load()`, etc.).

_Remember:_ evaluating Lua scripts with _Lua itself_ allows arbitrary code execution. ;)

Debug builds of `serde_luaq` should expect worse performance and memory usage than release builds,
which may exacerbate some issues.

## Reporting

Use GitHub's security tab to make a private report. Off-platform reports are not accepted.

- Reports **must** be written in English.

- Reports **must** include a proof of concept exploiting the bug in _this_ project, including
  the input Lua file that triggers the bug.

  _Ideally,_ the input Lua file should be able to trigger the bug using
  [the `lua_to_json` example][lua_to_json] with its default file size and table depth limits.

- Reports **must** be validated and verified by a human (you!) before submission.

  Properly triaging and investigating issues (not just security issues) takes time.

- [Writing a patch][write-pr] that fixes the issue would be appreciated. :)

- Proof-of-concept exploits, including input Lua files, are considered contributions to the project
  [made under its license][license], even if the report is not accepted as a security bug.

- Issues relating to external libraries and/or integration in other applications are not in scope
  for this project.

There are no bounties or payments on offer.

If you wish, we may credit you with discovery of the issue. The form of this credit is at my
personal discretion.

[latest]: https://github.com/micolous/serde_luaq/releases/
[license]: ./README.md#license
[lua_to_json]: ./serde_luaq/examples/lua_to_json.rs
[main-mem]: https://micolous.github.io/serde_luaq/serde_luaq/#memory-usage
[stable-mem]: https://docs.rs/serde_luaq/latest/serde_luaq/#memory-usage
[write-pr]: ./CONTRIBUTING.md#submitting-prs
