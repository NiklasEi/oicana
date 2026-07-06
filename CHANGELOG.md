# Changelog

## Upcoming

- Fix potential ABBA deadlock when doing template compilation in parallel to document exports
- Performance improvement: more finegrained internal locking to unlock more parallelization
- Bumped some dependencies to resolve security advisories
- Limit packed template archives to protect against zip bombs
  - Default limits: 10,000 entries and 512 MiB of decompressed content 

### Python integration
- release GIL where possible to allow parallelization

### PHP integration
- prevent panics across FFI border

### C# integration
- prevent panics across FFI border
- Fix non-ASCII string handling on Windows
- Fixed memory leak in input handing


## v0.4.0
- Update to Typst 0.15
  - It's now possible to export a combination of PDF standards
- Patched used Typst to support custom XMP metadata
  - This enables use-cases like e-invoices with factur-x/ZUGFeRD standards

### Python integration
- Lower the minimum supported Python version from 3.11 to 3.9
- Publish `musllinux` wheels (x86_64 and aarch64)


## v0.3.0
- previous `compile` methods in all integrations are now called `export`
- add new `compile` methods that return compiled documents, which can be
  - exported to any format
  - exported page by page
- templates and compiled documents offer `exportPdf`/`exportPng`/`exportSvg` convenience
  methods in all integrations
- compiled documents expose the compilation `warnings` in all integrations
- PDF tagging can be toggled via `tagged` in `[tool.oicana.export.pdf]` (defaults to `true`)


## v0.2.0
- fixed some edge-case panics is the central crates

### Rust integration v0.2.0
- re-export more Typst types

### CLI v0.2.0
- Compress images before writing them to disc (snapshots and comparison files)

### Typst package v0.2.0
- fix image helper for optional blob inputs

### Browser integration v0.2.0
- Expose getter for compilation warnings
- Move standard logs to trace
- Use explicit extensions in imports

### C# integration v0.2.0
- Expose getter for compilation warnings

### Java integration v0.2.0
- Expose getter for compilation warnings

### Node.js integration v0.2.0
- Expose getter for compilation warnings

### PHP integration v0.2.0
- Expose getter for compilation warnings

### Python integration v0.2.0
- Expose getter for compilation warnings




## Alpha

### CLI v0.1.0-alpha.17
- set document date on template scaffold

### CLI v0.1.0-alpha.16
- template now always exclude output and test directories
- validate command will ensure the given pdf standards for export are compatible
- `new` command

### Rust integration v0.1.0-alpha.12
<changes limited to internal crates for the CLI>

### C# integration v0.1.0-alpha.12
- `Oicana.Template` namespace was removed; all content moved to the root namespace

### Rust integration v0.1.0-alpha.11
- re-export all required oicana crates and Typst types
- more granular features in the integration and internal crates

### CLI v0.1.0-alpha.15
- Update to rust integration v0.1.0-alpha.11
- Show version of the bundled Typst compiler in version output

### Browser integration v0.1.0-alpha.8
- Use non deprecated parameter for WASM init call

### C# integration v0.1.0-alpha.11
- rename `ExportOptions` -> `ExportFormat` for consistency

### PHP integration v0.1.0-alpha.5
- Change pixels per pt default for png exports from 2 to 1 for consistency

### CLI v0.1.0-alpha.14 (WIP in 12/13)
- Update command when installing CLI via scripts

### CLI v0.1.0-alpha.11
### Rust integration v0.1.0-alpha.10
- watch mode for tests

### CLI v0.1.0-alpha.10
### Rust integration v0.1.0-alpha.9
### Typst integration v0.1.1
- Make inputs required by default with better error handling
- Support watch command in CLI
- Don't copy template dependencies on disk
- General CLI output improvements

### CLI v0.1.0-alpha.9
### Browser integration v0.1.0-alpha.7
### C# integration v0.1.0-alpha.10
### Node.js integration v0.1.0-alpha.8
### Python integration v0.1.0-alpha.5
### PHP integration v0.1.0-alpha.4
### Java integration v0.1.0-alpha.3
### Rust integration v0.1.0-alpha.8
- Add JSON input validation for inputs with schemas

### Browser integration v0.1.0-alpha.6
- return Uint8Array<ArrayBuffer> from compile methods
- Implement Disposable for automatic cleanup with `using`

### Node.js integration v0.1.0-alpha.7
- Implement Disposable for automatic cleanup with `using`
- more error handling
- expose cache eviction methods

### PHP integration v0.1.0-alpha.3
- expose cache eviction methods

### Python integration v0.1.0-alpha.4
- expose cache eviction methods
- rename `export_format` parameter to `export`

### Rust integration v0.1.0-alpha.7
- follow sym links when packing templates
- remove some possible panics

### CLI v0.1.0-alpha.8
- print packaged templates
- follow sym links when packing templates
- remove some possible panics

### C# integration v0.1.0-alpha.9
- Expose cache eviction on `Configuration` and `Template`

### Java integration v0.1.0-alpha.2
- improve overloads for Template#Compile

### C# integration v0.1.0-alpha.8
- Take inputs as dictionary
- Remove possible panics in error handling


### Browser integration v0.1.0-alpha.4
### Node.js integration v0.1.0-alpha.5
- Remove log statements

### C# integration v0.1.0-alpha.6
### Browser integration v0.1.0-alpha.3
### Node.js integration v0.1.0-alpha.4
- Unify function parameters
- Fix default compilation mode

### Crates v0.1.0-alpha.5
### CLI v0.1.0-alpha.6
### C# integration v0.1.0-alpha.5
### Browser integration v0.1.0-alpha.2
### Node.js integration v0.1.0-alpha.3
### Rust integration v0.1.0-alpha.5
- Fix template paths on Windows
- Update to Typst 0.14

### Browser v0.1.0-alpha.2
- improved Browser compatibility (e.g. works on newer Firefox Android now) 

### C# integration v0.1.0-alpha.4
- DO NOT USE; broken on Windows

### Crates v0.1.0-alpha.4 and CLI v0.1.0-alpha.5
- fix fuzzing tests from paths other than template root ((#28)[https://github.com/oicana/oicana/pull/20])

### Crates v0.1.0-alpha.3 and CLI v0.1.0-alpha.4
- mostly changes for CLI - v0.1.0-alpha.3

### CLI - v0.1.0-alpha.3
- fix CLI sometimes packaging test files ((#20)[https://github.com/oicana/oicana/pull/20])
- tests can be without a snapshot file ((#16)[https://github.com/oicana/oicana/pull/16])
- tests can fuzz json inputs with a json schema ((#16)[https://github.com/oicana/oicana/pull/16])
- tests will fail for missing snapshots ((#17)[https://github.com/oicana/oicana/pull/17])
- new options for test command `--update`/`-u` will overwrite/create snapshot files ((#17)[https://github.com/oicana/oicana/pull/17])
