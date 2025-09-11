== Development

As long as all defined inputs have default values, a template can be previewed by any Typst editor. That means
existing tools like the [official web app] or IDE plugins like [tinymist] can be used to develop an Oicana template.

The #link("https://github.com/oicana/oicana_example_templates")[`oicana_example_templates` repository on GitHub] includes instructions for some development setups.

=== Packing

During development, an Oicana template is a directory containing at least one Typst source file and a `typst.toml` manifest.
Before it can be compiled from Oicana integrations like the C#sym.hash or Node.js packages, the template has to be packed.

Packing is done via the Oicana CLI using the `pack` command. The output is a compressed file containing the template and all its dependencies.
