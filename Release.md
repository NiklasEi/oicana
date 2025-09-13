# Releasing Oicana

## CLI

The CLI is distributed through cargo-dist. Push a tag in the form of `oicana_cli-v0.0.0-alpha.1` to trigger the workflow.
It will build and package the CLI for all platforms currently set up.

The release pipeline is configured to run with latest stable rust.

### Updating dist

Run `dist init` to update the config and workflow.

## C#

The github workflow `.github/workflows/publish_csharp.yml` can be manually triggered.
It will build the native libraries for Linux, MacOS and Windows. Then all native libraries are
included in the C# package and bundled into a `.nupkg` which will be archived by the workflow.

The package can then be manually uploaded to the index.

### WASM

The CD workflow for WASM requires authentication to the npm registry. This is done by copying `ci.npmrc` into the
package directory before running `npm publish` and requires the environment variable `NPM_AUTH` to be set to base64
encoded basic auth credentials.
