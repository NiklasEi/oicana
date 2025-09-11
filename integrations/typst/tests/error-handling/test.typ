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
  "panicked with: \"The `manifest_version` 0 is not supported by this package. Please check if there is an update available!\"",
)
