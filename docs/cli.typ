#import "boxes.typ": *

= Oicana CLI<cli>

== Installation

#beta-note[There are no distributed builds of the CLI yet. You should have received a built or installer with this guide.

  With a clone of the repository and a working Rust setup, run `cargo install --path tools/oicana_cli` to install the CLI.
]

#link("https://github.com/oicana/oicana/releases")[Oicana releases on GitHub] include binaries and installers for the CLI.

== Usage

You can run `oicana -h` for a list of all commands and options.

=== Package a template

The command `oicana pack` can package an oicana template to be usable in all supported environments. It will bundle everything in an archive with fast compression.

While packing, all required dependencies will be copied from the local Typst cache. If a package from the `preview` namespace is missing, it will be downloaded from Typst universe. You can install packages with any namespace by copying them in the correct location (see @dependencies[Chapter]).

For a list of all command options, run `oicana pack -h`.

=== Testing

With `oicana test` all tests of the currently targeted template will be executed.

Learn more about testing Oicana templates in the @testing[testing section].

=== Validation

#note[The validation is currently work in progress. The command will only check if the manifest can be parsed.]

Oicana templates are valid Typst projects. Inputs require additional configuration in their `typst.toml`.
