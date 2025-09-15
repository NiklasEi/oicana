#import "/src/boxes.typ": *
#import "/src/docs-link.typ": *
#import "/src/code.typ": *


There are two types of inputs. A `json` input contains structured data while a `blob` input passes bytes and optionally metadata to the template. For example, in an invoice the items and customer data could be a `json` input and the company logo could be a `blob` input.

== The Oicana Typst package
Template inputs are configured in the manifest file `typst.toml`. The Oicana Typst package determines the current values of inputs.

\
#alpha-note[The Typst package is not (#link("https://github.com/typst/packages/pull/3058")[yet]) published on Typst universe. You will have to install it locally.

  1. Download and install #link("https://github.com/sjfhsjfh/typship/releases")[`the typship CLI`]
  2. Clone the Oicana repository
  3. Run `typship install local` in `oicana/integrations/typst`

  See #docs-link(<dependencies>, "../templates/dependencies.html")[the template dependencies section] for more information.]

\
Add the following to the top of your `main.typ` file to initialize the package:
#code("main.typ", ```typst
#import "@local/oicana:0.1.0": setup

#let read-project-file(path) = return read(path, encoding: none);
#let (input, oicana-image, oicana-config) = setup(read-project-file);
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
While developing an Oicana template in a Typst editor, it will be compiled in development mode. It makes sense to define `development` values for all required inputs of you template to have a functioning preview.

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
#import "@local/oicana:0.1.0": setup

#let read-project-file(path) = return read(path, encoding: none);
#let (input, oicana-image, oicana-config) = setup(read-project-file);

= Hello from Typst, #input.info.name

Now we can pass names into the template from any Oicana integration. We will set the name out of C#sym.hash in the next step.
```)

== Inputs in the C#sym.hash integration

With the input defined, we can update the packed template in the C#sym.hash project. Run `oicana pack` in the template directory and
replace `example-0.1.0.zip` in the ASP.NET project with the new file.

\
Our `compile` endpoint is currently calling `var stream = template.Compile([], [], CompilationOptions.Pdf());`. This compiles the template without any inputs. The first empty array are the `json` inputs and the second one the `blob` inputs.

\
Change the endpoint to set the name input we just defined.

#code(
  "Part of Program.cs",
  ```cs
  app.MapGet("compile", () =>
  {
      var input = new TemplateJsonInput("info", JsonSerializer.Deserialize<JsonNode>("{ \"name\": \"Baby Yoda\" }")!);
      var stream = template.Compile([input], [], CompilationOptions.Pdf());
      var now = DateTimeOffset.Now;
      // ... more code from before
  });
  ```,
)

\
Calling the endpoint now, will result in a PDF with the new name. Building on this minimal service, one could now start to accept inputs through the HTTP endpoint or add other inputs. Take a look at the open source #link("https://github.com/oicana/oicana-example-asp-net/")[ASP.NET example project on GitHub] for a more complete showcase of the Oicana C#sym.hash integration.
