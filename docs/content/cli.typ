#import "/src/boxes.typ": *
#import "/src/code.typ": *
#import "/src/constants.typ": *
#import "/src/docs-link.typ": *


#link(latest-cli)[CLI builds are published on GitHub]. You can pick and install the correct binary yourself or let a script do it for you.

\
Bash script:
#code("Script to install Oicana CLI", latest-cli-shell)

Powershell script:
#code("Powershell script to install Oicana CLI", latest-cli-powershell)

\
Run ```bash oicana -h``` for a list of all commands and options.

== Package a template

The command ```bash oicana pack``` can package an oicana template to be usable in all supported environments. It will bundle everything in an archive with fast compression.

While packing, all required dependencies will be copied from the local Typst cache. If a package from the `preview` namespace is missing, it will be downloaded from Typst universe. You can install packages with any namespace by copying them in the correct location (see #docs-link(<dependencies>, "./templates/dependencies.html")[the documentation on template dependencies]).

For a list of all command options, run ```bash oicana pack -h```.

== Testing

Example commands to test templates:
- ```bash oicana test``` - run all tests of the template in the current directory
- ```bash oicana test templates/invoice``` - run the tests of the template in the directory `templates/invoice`
- ```bash oicana test -a``` - run all tests of all templates found in the current directory and all child directories

Learn more about testing Oicana templates in the #docs-link(<testing>, "./templates/tests.html")[testing chapter].

== Compilation

For testing purposes, the CLI can compile not-packed Oicana templates. Inputs can be given as relative paths to files.

\
Example commands to compile templates:
- ```bash
  oicana compile -f pdf -j invoice=invoice.json -b logo=oicana.png```
  compile the template in the current directory to pdf with the given inputs.
- ```bash
  oicana compile templates/table -j input=templates/table/data.json```
  compile the template at `templates/table` to pdf with the given inputs.
- ```bash
  oicana compile templates/table -j input=templates/table/data.json -o out -n output.pdf```
  same as above, but with a custom output directory and output file name. The defaults are `output` and `{template}_{millies}.{format}` respectively.

== Validation

E.g. ```bash oicana validate templates/table``` will validate the table template.

\
If JSON inputs have schemas defined, the `validate` command will make sure that any default or development values are valid according to the schema.
