#import "/src/boxes.typ": *
#import "/src/constants.typ": *
#import "/src/docs-link.typ": *


#link(latest-cli)[CLI builds are published on GitHub]

Run `oicana -h` for a list of all commands and options.

== Package a template

The command `oicana pack` can package an oicana template to be usable in all supported environments. It will bundle everything in an archive with fast compression.

While packing, all required dependencies will be copied from the local Typst cache. If a package from the `preview` namespace is missing, it will be downloaded from Typst universe. You can install packages with any namespace by copying them in the correct location (see #docs-link(<dependencies>, "./oicana-templates/dependencies")[the documentation on template dependencies]).

For a list of all command options, run `oicana pack -h`.

== Testing

With `oicana test` all tests of the currently targeted template will be executed.

Learn more about testing Oicana templates in the #docs-link(<testing>, "./oicana-templates/tests")[testing section].

== Validation

#note[The validation is work in progress. The command will currently only check if the manifest can be parsed.]

Oicana templates are valid Typst projects. Inputs require additional configuration in their `typst.toml`.
