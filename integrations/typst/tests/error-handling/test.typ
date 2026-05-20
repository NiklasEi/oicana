#import "../../src/lib.typ": setup

/// Tries to read manifest file first
#let no-manifest(path) = {
  assert.eq(path, "typst.toml")
  return read("non-existing-file.toml")
}
#let error = catch(() => setup(no-manifest));
#assert(error.starts-with("file not found"))

/// Complains about missing oicana section in manifest
#let no-manifest(path) = {
  assert.eq(path, "typst.toml")
  return read("empty.toml", encoding: none)
}
#let error = catch(() => setup(no-manifest));
#assert.eq(
  error,
  "panicked with: \"This Typst project is not an Oicana template. Please add a `[tool.oicana]` section in your `typst.toml` file.\"",
)

/// Complains about missing manifest_version in manifest
#let no-manifest(path) = {
  assert.eq(path, "typst.toml")
  return read("no-manifest-version.toml", encoding: none)
}
#let error = catch(() => setup(no-manifest));
#assert.eq(
  error,
  "panicked with: \"The `[tool.oicana]` section has to contain a `manifest_version`.\"",
)

/// Complains about unsupported manifest_version
#let no-manifest(path) = {
  assert.eq(path, "typst.toml")
  return read("wrong-manifest-version.toml", encoding: none)
}
#let error = catch(() => setup(no-manifest));
#assert.eq(
  error,
  "panicked with: \"The `manifest_version` 0 is not supported by this package version. Please check for updates at https://typst.app/universe/package/oicana\"",
)

/// Complains when `tool` is not a dictionary
#let tool-not-dict(path) = {
  assert.eq(path, "typst.toml")
  return read("tool-not-dictionary.toml", encoding: none)
}
#let error = catch(() => setup(tool-not-dict));
#assert.eq(
  error,
  "panicked with: \"This Typst project is not an Oicana template. Please add a `[tool.oicana]` section in your `typst.toml` file.\"",
)

/// Complains when `tool` exists but has no `oicana` section
#let tool-without-oicana(path) = {
  assert.eq(path, "typst.toml")
  return read("tool-without-oicana.toml", encoding: none)
}
#let error = catch(() => setup(tool-without-oicana));
#assert.eq(
  error,
  "panicked with: \"This Typst project is not an Oicana template. Please add a `[tool.oicana]` section in your `typst.toml` file.\"",
)

/// Complains when `manifest_version` is not an integer
#let string-manifest-version(path) = {
  assert.eq(path, "typst.toml")
  return read("string-manifest-version.toml", encoding: none)
}
#let error = catch(() => setup(string-manifest-version));
#assert.eq(
  error,
  "panicked with: \"The `[tool.oicana]` section has to contain a `manifest_version`.\"",
)

/// Complains when an input has an unknown `type`
#let unknown-input-type(path) = {
  assert.eq(path, "typst.toml")
  return read("unknown-input-type.toml", encoding: none)
}
#let error = catch(() => setup(unknown-input-type));
#assert.eq(
  error,
  "panicked with: \"Found unknown input type 'xml'. Should be \\\"json\\\" or \\\"blob\\\".\"",
)
