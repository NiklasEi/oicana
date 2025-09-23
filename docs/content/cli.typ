#import "/src/boxes.typ": *
#import "/src/constants.typ": *
#import "/src/docs-link.typ": *


#link(latest-cli)[CLI builds are published on GitHub]. You can pick and install the correct binary yourself or let a script do it for you.

Shell script: #latest-cli-shell

Powershell script: #latest-cli-powershell

\
Run `oicana -h` for a list of all commands and options.

== Package a template

The command `oicana pack` can package an oicana template to be usable in all supported environments. It will bundle everything in an archive with fast compression.

While packing, all required dependencies will be copied from the local Typst cache. If a package from the `preview` namespace is missing, it will be downloaded from Typst universe. You can install packages with any namespace by copying them in the correct location (see #docs-link(<dependencies>, "./templates/dependencies.html")[the documentation on template dependencies]).

For a list of all command options, run `oicana pack -h`.

== Testing

With `oicana test` all tests of the currently targeted template will be executed.

Learn more about testing Oicana templates in the #docs-link(<testing>, "./templates/tests.html")[testing section].

== Validation

#note[The validation is work in progress. The command will currently only check if the manifest can be parsed.]

Oicana templates are valid Typst projects. Inputs require additional configuration in their `typst.toml`.

== Compilation

For testing purposes, the CLI can compile not-packed Oicana templates. Inputs can be given as relative paths to files.
