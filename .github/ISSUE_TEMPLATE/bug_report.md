---
name: Bug report
about: Report a bug with the code in this repository
title: "[BUG] "
labels: bug
assignees: ''

---

## Describe the bug
A clear and concise description of what the bug is.

## Code to reproduce
Attach or link to your Rust parsing code and/or `Deserialize` structs here.

## Expected behaviour
A clear and concise description of what you expected to happen.

## Actual behaviour
What happened instead?

## Logs / screenshots
If applicable, add logs or screenshots to help explain your problem.

## Sample input Lua file
If you're reporting a bug in the Lua parser, you **must** attach, copy or link to a Lua file you're
trying to parse that reproduces this issue. If your Lua file contains binary data, put it in a ZIP
file before uploading.

Make sure to not include any personal information, as anything posted here is publicly accessible.

## Platform

- `serde_luaq` version or git commit:
- Host OS and CPU architecture:
- Target OS and CPU architecture (if different from host):
- How was the file Lua file generated?
- What is at least _one_ version of Lua which can parse your input file correctly?
- (Was that version/were those versions) of Lua released on/after 2020-01-01?
- What is your system's locale / language set to?

### Output of `rustc -vV`:

```

```

## Additional context
Add any other context about the problem here.
