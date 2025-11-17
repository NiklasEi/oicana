#import "/src/boxes.typ": *
#import "/src/docs-link.typ": *
#import "/src/code.typ": *


There are two types of inputs. A `json` input contains structured data while a `blob` input passes bytes and optionally metadata to the template. For example, in an invoice the items and customer data could be a `json` input and the company logo could be a `blob` input.

== The Oicana Typst package
Template inputs are configured in the manifest file `typst.toml`. The Oicana Typst package determines the current values of inputs.

\
Add the following to the top of your `main.typ` file to initialize the package:
#code("main.typ", ```typst
#import "@preview/oicana:0.1.0": setup

#let read-project-file(path) = return read(path, encoding: none);
#let (input, oicana-image, oicana-config) = setup(read-project-file);

#set document(date: datetime.today())
```)
\
This snippet gives the Oicana package access to the Typst project's files. We can now use the return values from calling `setup` in the rest of the template.

== Defining inputs

We will use a `json` input to pass a name into the template. Add the following to the end of the `typst.toml` file:
#code("typst.toml", ```toml
[[tool.oicana.inputs]]
type = "json"
key = "info"
```)
\
The value of this input is now available in the template as `input.info`, where `info` is the key of the input as defined in `typst.toml`.

\
While we develop the template, the value of the input will be `none`, because there is no Oicana integration setting a value for it. We can change that by defining a `default` or `development` value for the input.

=== Default and Development values

Inputs can define two different fallback values, `default` and `development`. These fallback values differ in priority based on which mode the template is compiled in.

\
When compiling a template in development mode, input values have the priority

1. Explicit input value (for example through an integration)
2. `development` value
3. `default` value

\
If you compile in production mode, the `development` value is ignored:

1. Explicit input value (for example through an integration)
2. `default` value

\
#note[
  *When to use each mode:*

  *Development mode* is used in two scenarios:
  - When developing templates in a Typst editor to see live previews with test data or using other tooling without Oicana integration (like the official Typst CLI)
  - By default during template registration (typically at server startup) to warm up the Typst compilation cache

  *Production mode* is the default for document compilation in integrations. It ensures your application fails explicitly rather than accidentally using test data in production documents. If an input value is missing in production mode and the input does not have a default value, the compilation will fail unless your template handles `none` values for that input.
]

\
While developing an Oicana template in a Typst editor, it will be compiled in development mode. It makes sense to define `development` values for all required inputs of your template to have a functioning preview.

\
Let's extend our input with a `development` value. First create an `info.json` file in the template directory:

#code("info.json", ```json
{
  "name": "Chuck Norris"
}
```)
\
Then extend the input definition and set the `development` value to be `info.json`:
#code("typst.toml", ```toml
[[tool.oicana.inputs]]
type = "json"
key = "info"
development = "info.json"
```)
\
In our template we can now use `input.info.name` and the preview will show "Chuck Norris".
#code("main.typ", ```typst
#import "@preview/oicana:0.1.0": setup

#let read-project-file(path) = return read(path, encoding: none);
#let (input, oicana-image, oicana-config) = setup(read-project-file);

= Hello from Typst, #input.info.name

Now we can pass names into the template from any Oicana integration.
```)

\
With the input defined in your template, you're ready to choose an integration and learn how to pass dynamic values from your application code.
