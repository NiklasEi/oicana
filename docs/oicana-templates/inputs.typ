#import "../src/boxes.typ": *
#import "../src/code.typ": *

== Inputs

Oicana supports two types of inputs. A `json` input takes structured data while binary data can be passed into templates through a `blob` input.

Inputs are defined in the template manifest. Integrations can list all inputs of a template to, for example, validate input values or offer an editor.

=== `json` inputs

The `type` property of the input definition should be `json`. The only other required property is `key`.

#local-code("Part of typst.toml", "templates-input-json")[
  ```toml
  [[tool.oicana.inputs]]
  type = "json"
  key = "data"
  ```]

To configure default or development values for the input, include json files in the template and point to them in the input definition. A json schema file can be used for input validation.

#alpha-note(
  "The json schema validation is not complete yet. You can set the property and maintain the schema, but at the moment it is ignored by Oicana.",
)

#local-code(
  "Part of typst.toml",
  "templates-input-json-default-schema-development",
)[
  ```toml
  [[tool.oicana.inputs]]
  type = "json"
  key = "data"
  default = "data.json"
  schema = "data.schema.json"
  development = "data.json"
  ```]

=== `blob` inputs

`blob` inputs can be used for binary data like images. Additional metadata can be used to further specify the type of binary data in the input.


#local-code("Part of typst.toml", "templates-input-blob")[
  ```toml
  [[tool.oicana.inputs]]
  type = "blob"
  key = "logo"
  ```]

As a common use case for `blob` inputs, images have special support in the `oicana` Typst package.
