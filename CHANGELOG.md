# Next release


# Browser integration

- return Uint8Array<ArrayBuffer> from compile methods
- Implement Disposable for automatic cleanup with `using`

# Node.js integration

- Implement Disposable for automatic cleanup with `using`


# Java integration v0.1.0-alpha.2
- improve overloads for Template#Compile

# C# integration v0.1.0-alpha.8

- Take inputs as dictionary
- Remove possible panics in error handling


# Browser integration v0.1.0-alpha.4
# Node.js integration v0.1.0-alpha.5

- Remove log statements

# C# integration v0.1.0-alpha.6
# Browser integration v0.1.0-alpha.3
# Node.js integration v0.1.0-alpha.4

- Unify function parameters
- Fix default compilation mode

# Crates v0.1.0-alpha.5
# CLI v0.1.0-alpha.6
# C# integration v0.1.0-alpha.5
# Browser integration v0.1.0-alpha.2
# Node.js integration v0.1.0-alpha.3
# Rust integration v0.1.0-alpha.5

- Fix template paths on Windows
- Update to Typst 0.14

## Browser v0.1.0-alpha.2

- improved Browser compatibility (e.g. works on newer Firefox Android now) 

# C# integration v0.1.0-alpha.4
- DO NOT USE; broken on Windows

# Crates v0.1.0-alpha.4 and CLI v0.1.0-alpha.5

- fix fuzzing tests from paths other than template root ((#28)[https://github.com/oicana/oicana/pull/20])

# Crates v0.1.0-alpha.3 and CLI v0.1.0-alpha.4

- mostly changes for CLI - v0.1.0-alpha.3

# CLI - v0.1.0-alpha.3

- fix CLI sometimes packaging test files ((#20)[https://github.com/oicana/oicana/pull/20])
- tests can be without a snapshot file ((#16)[https://github.com/oicana/oicana/pull/16])
- tests can fuzz json inputs with a json schema ((#16)[https://github.com/oicana/oicana/pull/16])
- tests will fail for missing snapshots ((#17)[https://github.com/oicana/oicana/pull/17])
- new options for test command `--update`/`-u` will overwrite/create snapshot files ((#17)[https://github.com/oicana/oicana/pull/17])
