# Releasing Oicana

1. Bump general crate version in `cargo.toml`
2. Bump CLI version
3. Integrations
  * bump oicana_browser_wasm in integrations/browser/oicana_browser_wasm/Cargo.toml
  * bump C# integration in integrations/csharp/Oicana/Oicana.csproj and integrations/csharp/oicana_csharp/Cargo.toml
  * bump Node.Js in integrations/node/oicana-node-native/Cargo.toml and integrations/node/oicana-node-native/package.json
    * run `yarn build` and `yarn format`
  * Python:
    * native: integrations/python/oicana-python-native/pyproject.toml and integrations/python/oicana-python-native/Cargo.toml
    * wrapper: integrations/python/oicana-python/pyproject.toml
4.
  * tag CLI version => will trigger CD pipeline; publish to crates.io manually
  * tag crates version => manual publish
  * tag C# version => will trigger CD pipeline
  * tag browser => trigger CD pipeline `publish npm @oicana/browser-wasm`
  * tag python => will trigger CD pipeline
5.
  * bump Node.Js wrapper integrations/node/oicana-node/package.json
    * `npm i` after bumping dependency
  * bump browser wrapper integrations/browser/oicana-browser/package.json
    * `npm i` after bumping dependency


## CLI

The CLI is distributed through cargo-dist. Push a tag in the form of `oicana_cli-v0.0.0-alpha.1` to trigger the workflow.
It will build and package the CLI for all platforms currently set up.

The release pipeline is configured to run with latest stable rust.

Remember to publish the CLI to crates.io as well for proper `cargo binstall` support (see "Crates" section below).

### Updating dist

Run `dist init` to update the config and workflow.

## C#

The github workflow `.github/workflows/publish_csharp.yml` can be manually triggered or by pushign a tag in the form of `oicana_csharp-v[0-9]+.[0-9]+.[0-9]+*`.
It will build the native libraries for Linux, MacOS and Windows. Then all native libraries are
included in the C# package and bundled into a `.nupkg` which will be archived by the workflow.

The pipeline pushes the new version to nuget.

## WASM

The WASM integration is published in two steps:

1. `oicana_browser_wasm`
  - Bump version in `integrations/browser/oicana_browser_wasm/Cargo.toml`
  - Trigger `.github/workflows/publish-integration-browser-wasm.yml`
2. `oicana-browser`
  - Bump `@oicana/browser-wasm` dependency
  - Bump version
  - Run `npm i` to update lock file
  - Trigger `.github/workflows/publish-integration-browser.yml`

### Pipeline authentication

The CD workflow for WASM requires authentication to the npm registry. This is done by copying `ci.npmrc` into the
package directory before running `npm publish` and requires the environment variable `NPM_AUTH` to be set to base64
encoded basic auth credentials.

## Node


The Node integration is published in two steps, first `@oicana/node-native` then `@oicana/node`.

Bump the version of `@oicana/node-native` in `integrations/node/oicana-node-native/Cargo.toml` and `integrations/node/oicana-node-native/package.json`
then rebuild the bindings with `yarn build` and format them via `yarn format`.

The CD pipeline for `@oicana/node-native` runs on merges to main. It will publish the package if the current version doesn't exist on the index yet.

The wrapper package `@oicana/node` is currently published manually.

## Crates

All rust crates excluding the native wrappers are published to crates.io. This includes the CLI tools `oicana_cli` and `test_compare`.

Run `cargo publish --workspace --dry-run` to try publishing all crates.
