#import "../src/boxes.typ": *

== Dependencies<dependencies>

An Oicana template can use any Typst package. Public packages can be found in #link("https://typst.app/universe")[the Typst Universe]. You can also install private packages locally and use them in Oicana templates. Using dependencies works just as for any other Typst document.

#example[Take a look at the #link("https://github.com/oicana/oicana-example-templates/tree/main/templates/dependency")[example Oicana template `dependency`]. It uses the awesome #link("https://typst.app/universe/package/cetz/")[`cetz` package to draw a diagram] based on an input.]

=== Local packages

A locally installed package can have any namespace. A common one is `@local`, but feel free to use your company name or any other identifier.

To install a local Typst package, you can use a community developed tool or manually copy files to the right place.

==== Typship

#link("https://github.com/sjfhsjfh/typship")[Typship] is a tool for Typst package development and publishing. Its CLI can install local Typst packages for you.

To install a package into the `@local` namespace, run `typship install local` in the package directory.

==== Manual

Installing a Typst package means that Typst can find it at `{data-dir}/typst/packages/{namespace}/{name}/{version}`. Here, `{data-dir}` is

- `XDG_DATA_HOME` or `~/.local/share` on Linux
- `~/Library/Application Support` on macOS
- `%APPDATA%` on Windows

For example, on Linux:

1. Store a package in `~/.local/share/typst/packages/local/my-package/1.0.0`
2. Import all items from the package with `#import "@local/my-package:1.0.0": *` in a Typst document

For more information on packages, please refer to #link("https://github.com/typst/packages")[Typst's packages repository].
