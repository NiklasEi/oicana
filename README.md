# Oicana
*Dynamic PDF Generation based on Typst*

https://oicana.com

Oicana offers seamless PDF templating across multiple platforms. Define your templates in Typst, specify dynamic inputs, and generate high quality PDFs from any environment - whether it's a web browser, server application, or desktop software.

> **Oicana is in Alpha! It is rough around the edges and has a limited number of integrations.**

## What Oicana offers

- *Multi-platform* - The same templates work with all Oicana integrations.
- *Powerful Layouting* - Templates can use all of Typst's functionality, including its extensive package ecosystem.
- *Performant* - Create a PDF in single digit milliseconds.
- *AI and Version Control Ready* - Templates are text files. They can live next to your code and AI can assist in writing them.
- *Escape Vendor Lock-in* - Reuse templates with other Typst based solutions. The Typst compiler is open source!

## Getting started

The [getting started guide][getting-started] demonstrates how to
1. Create an Oicana Template
2. Define and use inputs for the template
3. Compile a PDF based on the template from either a
   * C# application using [ASP.NET](https://dotnet.microsoft.com/en-us/apps/aspnet)
   * Rust application with [axum](https://github.com/tokio-rs/axum)
   * Node.js application using [NestJs](https://nestjs.com/)
   * Python application with [FastAPI](https://fastapi.tiangolo.com/)

## Available integrations

Integrations allow you to use Oicana templates from different platforms and programming languages.

Ready to use:
* TypeScript/JavaScript
    * in the browser -> [@oicana/browser](https://www.npmjs.com/package/@oicana/browser)
    * Node.js -> [@oicana/node](https://www.npmjs.com/package/@oicana/node)
* C# -> [Oicana](https://www.nuget.org/packages/Oicana)
* Rust -> [oicana](https://crates.io/crates/oicana)
* Python -> [oicana](https://pypi.org/project/oicana/)

You can find an open source example project in the [Oicana GitHub organization][oicana-github] for every available integration.
The example project for the browser integration is deployed to https://example.oicana.com (not compatible with some mobile browsers).

> More integrations are planned. If you are missing a specific one, please open a GitHub issue or sent us an e-mail at `hello@oicana.com`. This helps with prioritizing.

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

### Testing

Snapshot tests can be defined for every template. The CLI described in the next section has an `oicana test` command.

### CLI

Please refer to the documentation for installation instructions: https://docs.oicana.com/cli

#### Packaging

Packing Oicana templates is required to use templates with integrations.

Example commands to package templates:
- `oicana pack --all` - pack all templates in the current directory (including child directories)
- `oicana pack templates/invoice -o output` - pack the template in the `templates/invoice` directory and put the output into the `output` directory


#### Compilation

The `compile` command will create `pdf`, `png`, or `svg` files from unpacked templates.

Example commands to compile templates:
- `oicana compile templates/invoice -f pdf -j invoice=templates/invoice/invoice.json -b logo=templates/invoice/logo.jpg`
- `oicana compile templates/table -j input=templates/table/data.json`
- `oicana compile templates/package -j input=templates/package/sample.json`


#### Test

Example commands to test templates:
- `oicana test` - run all tests of the template in the current directory
- `oicana test templates/invoice` - run the tests of the template in the directory `templates/invoice`
- `oicana test -a` - run all tests of all templates found in the current directory and all child directories


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
[oicana-example-templates]: https://github.com/oicana/oicana-example-templates
[napi]: https://napi.rs/
[oicana-typst]: https://typst.app/universe/package/oicana
[docs]: https://docs.oicana.com
[getting-started]: https://docs.oicana.com/getting-started
