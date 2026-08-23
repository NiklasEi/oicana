# Oicana

*Turn a Typst project into a document template your software application can render.*

This package is the template-side half of [Oicana](https://oicana.com). It collects the inputs your application passes in and falls back to the `default` or `development` values from the manifest when they are missing, so one file serves both as a live preview in a Typst editor and as a production document rendered from Node.js, Python, Java, C#, Rust, PHP, or the browser.

> This package is [MIT licensed](./LICENSE.md) and free to use. The Oicana integrations that render these templates from application code are source-available: free for noncommercial use, with commercial use free for 30 days. See [pricing](https://oicana.com/#pricing).

## Setup

Every template imports `setup` and destructures its return values:

```typst
#import "@preview/oicana:0.2.0": setup

#let read-project-file(path) = read(path, encoding: none)
#let (input, oicana-image, oicana-config) = setup(read-project-file)
```

`read-project-file` is passed in because a package cannot read the files of the project using it. `input` holds the JSON inputs, `oicana-image` resolves blob inputs to images, and `oicana-config` exposes the compilation configuration.

## Example

A `typst.toml` declaring one JSON and one blob input:

```toml
[package]
name = "example"
version = "0.1.0"
entrypoint = "main.typ"

[tool.oicana]
manifest_version = 1

[[tool.oicana.inputs]]
type = "json"
key = "data"
development = "data.json"

[[tool.oicana.inputs]]
type = "blob"
key = "logo"
development = { file = "company-logo.png" }
```

The matching `main.typ`:

```typst
#import "@preview/oicana:0.2.0": setup

#let read-project-file(path) = read(path, encoding: none)
#let (input, oicana-image, oicana-config) = setup(read-project-file)

#set document(date: datetime.today())

The current value of the input with the key "data":
#input.data

The image passed into the template with the input key "logo": \
#oicana-image("logo")
```

Previewing this in a Typst editor shows the contents of `data.json` and `company-logo.png`. Rendered through an Oicana integration (from C# code, say), the application's values are used instead.

A Typst project that configures Oicana in its manifest and uses this package is what the documentation calls an *Oicana template*.

## Why Oicana

- **Runs in your infrastructure**: PDFs are generated inside your own application. No data is sent to a third-party service.
- **Multi-platform**: the same template works in the browser, Node.js, C#, Java, Rust, Python, and PHP.
- **Powerful layouting**: templates have all of Typst, including its package ecosystem.
- **Performant**: a warmed up template renders a PDF in single-digit milliseconds.
- **AI and version control ready**: templates are text files. They live next to your code, and AI can help write them.
- **No proprietary format**: templates are plain Typst projects. The Typst compiler is open source.

## Where to go next

- [Getting started](https://oicana.com/docs/getting-started/1-setup/): build a template and render it from your language of choice
- [Inputs](https://oicana.com/docs/templates/inputs/): JSON schemas, blob metadata, defaults, and required inputs
- [Example templates](https://github.com/oicana/oicana-example-templates): including a complete e-invoice
- [The Oicana CLI](https://oicana.com/docs/cli/): scaffold, validate, snapshot-test, and pack templates

## Licensing

This package is available under the [MIT license](./LICENSE.md).

Oicana itself is source-available under the [PolyForm Noncommercial License 1.0.0](https://github.com/oicana/oicana/blob/main/LICENSE.md). For commercial licensing details, see [the Oicana website](https://oicana.com/#pricing).
