#import "/src/docs-link.typ": *



An Oicana template is a Typst project that can define dynamic inputs. A template can be compiled with given inputs out of different programming languages through Oicana integrations.

\
You can find some #link("https://github.com/oicana/oicana-example-templates")[example Oicana templates on GitHub].

== Development

As long as all defined inputs have default values, a template can be previewed by any Typst editor. That means
existing tools like the #link("https://typst.app/")[official web app] or IDE plugins like #link("https://marketplace.visualstudio.com/items?itemName=myriad-dreamin.tinymist")[tinymist]
can be used to develop Oicana templates.

\
The #docs-link(<getting-started-setup>, "./getting-started/1-setup.html")[getting started section includes instructions for a development setup].

== Packing

During development, an Oicana template is a directory containing at least one Typst source file and a `typst.toml` manifest.
Before it can be compiled from Oicana integrations like the NuGet or NPM packages, the template has to be packed.

\
Packing is done via the Oicana CLI using the `pack` command. The output is a compressed file containing the template and all its dependencies.
