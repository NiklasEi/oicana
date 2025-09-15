#import "/src/boxes.typ": *
#import "/src/code.typ": *

== Inputs

Oicana supports two types of inputs. A json input takes structured data while binary data can be passed into templates through a blob input.

\
Inputs are defined in the template manifest. Integrations can list all inputs of a template to, for example, validate input values or offer an editor.

=== Json inputs

The `type` property of the input definition must be `json`. The only other required property is `key`.

#code("Part of typst.toml", "templates-input-json")[
  ```toml
  [[tool.oicana.inputs]]
  type = "json"
  key = "data"
  ```]

\
A json schema file can be used for input validation.

#alpha-note(
  "The json schema validation is not complete yet. You can set the property and maintain the schema, but at the moment it is ignored by Oicana.",
)

#code(
  "Part of typst.toml",
  "templates-input-json-default-schema-development",
)[
  ```toml
  [[tool.oicana.inputs]]
  type = "json"
  key = "data"
  schema = "data.schema.json"
  ```]

=== Blob inputs

Blob inputs can be used for binary data like images. Additional metadata can be used to further specify the type of binary data in the input.


#code("Part of typst.toml", "templates-input-blob")[
  ```toml
  [[tool.oicana.inputs]]
  type = "blob"
  key = "logo"
  ```]

As a common use case for blob inputs, images have special support in the `oicana` Typst package.

== Default and Development values

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
Considering a template with the files `development-data.json`, `default-data.json`, `development-logo.png`, and `default-logo.png`. It could define the following inputs:

#code("Part of typst.toml", "templates-input-defaults")[
  ```toml
  [[tool.oicana.inputs]]
  type = "json"
  key = "data"
  development = "development-data.json"
  default = "default-data.json"

  [[tool.oicana.inputs]]
  type = "blob"
  key = "logo"
  development = { file = "development-logo.png", meta = { image_format = "png", foo = 5, bar = ["development", "two"] } }
  default = { file = "default-logo.png", meta = { image_format = "png", foo = 5, bar = ["default", "two"] } }
  ```]
_The `default.meta` objects for blob fallback values are optional._

\
In the preview of an editor, the content of `development-data.json` and `development-logo.png` would be used. If compiled in production mode through an Oicana integration, the default fallbacks would be used if the input values are not set programmatically.
