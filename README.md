# Oicana
*Dynamic PDF Generation based on Typst*

https://oicana.com

Oicana offers seamless PDF templating across multiple platforms. Define your templates in Typst, specify dynamic inputs, and generate high quality PDFs from any environment - whether it's a web browser, server application, or desktop software.

> **Oicana is in Alpha! It is rough around the edges and has a limited number of integrations.**

## What Oicana offers

- *Multi-platform* - The same templates work with all Oicana integrations.
- *Powerful Layouting* - Templates can use all of Typst's functionality including its extensive package ecosystem.
- *Performant* - Create a PDF in single digit milliseconds.
- *Version Control Ready* - Templates are mostly text files and can live next to your source code.
- *Escape Vendor Lock-in* - Reuse templates with other Typst based solutions. The Typst compiler is open source!

## Getting started

The [getting started guide][getting-started] demonstrates how to
1. Create an Oicana Template
2. Compile a PDF based on the template from an ASP.NET application
3. Define and use inputs for the template

## Available integrations

Integrations allow you to use Oicana templates from different platforms and programming languages.

Ready to use:
* TypeScript/JavaScript in the browser
* C#

Work in progress:
* Rust
* TypeScript/JavaScript in Node.js

You can find an open source example project in the [Oicana GitHub organization][oicana-github] for every available integration.
The example project for the browser integration is deployed to https://example.oicana.com (not mobile friendly).

> More integrations are planned. If you are missing something, please open a GitHub issue or sent us an e-mail at `hello@oicana.com`. This helps with prioritizing.

## Oicana template development

For more details, please take a look at [the documentation][docs].

An Oicana template consists of
- one or multiple Typst `.typ` files
- a `typst.toml` manifest with
  - `name`, `version`, `entrypoint` and `tool.oicana.manifest_version` values
  - any number of input definitions

Oicana templates are "normal" Typst projects and can be worked on in the official [Typst editor][Typst] or any other editor with Typst support.

You can find a [couple of open source example templates][oicana-example-templates] on GitHub.

### Typst package

The Oicana Typst package has to be set up for every template. It will handle inputs for you and fall back on their default or development values when needed.

The package needs minimal setup:
```typst
#import "@preview/oicana:0.1.0": setup

#let read-project-file(path) = return read(path, encoding: none);
#let (input, oicana-image, oicana-config) = setup(read-project-file);
```

[The example templates][oicana-example-templates] showcase how to use the return values of the setup function.

> The package is not yet published in the Typst universe. You can install the package locally by running [`typship install local`][typship] in `./integrations/typst`.

### Testing

Snapshot tests can be defined for every template. The CLI described in the next section has an `oicana test` command.

### CLI

> There are no binaries of the CLI published yet. Install the current CLI with `cargo install --path tools/oicana_cli` from a clone of this repository.

You can find more information on the CLI in the documentation and with the `oicana help` command.

#### Packaging

Packing Oicana templates is required to use templates with integrations.

Example commands to package templates:
- `oicana pack --all` - pack all templates in the current directory (including child directories)
- `oicana pack templates/invoice -o output` - pack the template in the `templates/invoice` directory and put the output into the `output` directory


#### Compilation

The `compile` command will create `pdf`, `png`, or `svg` files from unpacked templates.

Example commands to compile templates:
- `oicana compile templates/invoice -f pdf -j invoice=templates/invoice/invoice.json -b logo=templates/invoice/logo.jpg`
- `oicana compile templates/test -j input=templates/test/sample.json`
- `oicana compile templates/package -j input=templates/package/sample.json`


#### Test

Example commands to test templates:
- `oicana test` - run all tests of all templates in the current directory (including child directories)
- `oicana test templates/invoice` - run the tests of the template in the directory `templates/invoice`


## Pronunciation
/ɔɪkɑna/

## Licensing

Oicana is source-available under [PolyForm Noncommercial License 1.0.0](./LICENSE.md). You can use it for free in any noncommercial context.

If you would like to use Oicana in a commercial capacity, please contact us at `hello@oicana.com`.

The [Typst integration][oicana-typst] and several example projects in the Oicana GitHub organization are open source under their respective licenses.
The example projects depend on Oicana integrations that are licensed under the before mentioned [PolyForm Noncommercial License 1.0.0](./LICENSE.md).


See [NOTICE](NOTICE) for a report of licenses in this project.



[Typst]: https://typst.app/
[typst-universe]: https://typst.app/universe/
[typst-packages]: https://github.com/typst/packages/
[oicana-github]: https://github.com/oicana
[oicana-example-templates]: https://github.com/oicana/oicana_example_templates
[typship]: https://github.com/sjfhsjfh/typship
[napi]: https://napi.rs/
[oicana-typst]: https://typst.app/universe/package/oicana
[docs]: https://docs.oicana.com
[getting-started]: https://docs.oicana.com/getting-started
