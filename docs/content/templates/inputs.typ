#import "/src/boxes.typ": *
#import "/src/code.typ": *

== Inputs

Oicana supports two types of inputs. A JSON input takes structured data while binary data can be passed into templates through a blob input.

\
Inputs are defined in the template manifest. Integrations can list all inputs of a template to, for example, validate input values or offer an editor.

=== JSON inputs

The `type` property of the input definition must be `json`. The only other required property is `key`.

#code("Part of typst.toml", ```toml
[[tool.oicana.inputs]]
type = "json"
key = "data"
```)

\
A JSON schema file can be used for input validation. When a schema is defined, Oicana validates JSON inputs against it before compilation. Invalid inputs are rejected with detailed error messages pointing to the specific fields that failed validation.

#code(
  "Part of typst.toml",
  ```toml
  [[tool.oicana.inputs]]
  type = "json"
  key = "data"
  schema = "data.schema.json"
  ```,
)

\
To disable validation for an individual input while keeping the schema (e.g. for fuzzing only), set `validate` to `false`:

#code(
  "Part of typst.toml",
  ```toml
  [[tool.oicana.inputs]]
  type = "json"
  key = "data"
  schema = "data.schema.json"
  validate = false
  ```,
)

When `validate` is `false`, no validator is compiled for this input during template initialization, even if a schema is defined. The schema can still be used by the test runner for fuzzing. `validate` defaults to `true`.

=== Blob inputs

Blob inputs can be used for binary data like images. Additional metadata can be used to further specify the type of binary data in the input.


#code("Part of typst.toml", ```toml
[[tool.oicana.inputs]]
type = "blob"
key = "logo"
```)

As a common use case for blob inputs, images have special support in the `oicana` Typst package.

== Default and Development values

Inputs can define two different fallback values, `default` and `development`.

\
When compiling a template in development mode, input values have the priority

1. Explicit input value
2. `development` value
3. `default` value

\
If you compile in production mode, the `development` value is ignored:

1. Explicit input value
2. `default` value

\
While developing an Oicana template in a Typst editor, it will be compiled in development mode. It makes sense to define `development` values for all required inputs of you template to have a functioning preview.

\
Considering a template with the files `development-data.json`, `default-data.json`, `development-logo.png`, and `default-logo.png`. It could define the following inputs:

#code("Part of typst.toml", ```toml
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
```)
_The `default.meta` objects for blob fallback values are optional._

\
In the preview of an editor, the content of `development-data.json` and `development-logo.png` would be used. If compiled in production mode through an Oicana integration, the default fallbacks would be used if the input values are not set programmatically.

== Validation configuration

By default, all JSON inputs with a schema are validated before compilation. You can control this at two levels.

=== Per-template default

The `validate_json_inputs_by_default` property in `[tool.oicana]` controls whether validation is enabled when a template is loaded. It defaults to `true`. Setting it to `false` means the template starts with validation disabled, though integrations can still enable it at runtime per template instance.

#code("Part of typst.toml", ```toml
[tool.oicana]
manifest_version = 1
validate_json_inputs_by_default = false
```)

=== Per-input opt-out

Each JSON input has an optional `validate` property that defaults to `true`. Setting it to `false` prevents Oicana from compiling a validator for that input, even if a schema is defined. This is useful when a schema is only needed for test fuzzing and not for runtime validation.

#code("Part of typst.toml", ```toml
[[tool.oicana.inputs]]
type = "json"
key = "data"
schema = "data.schema.json"
validate = false
```)

Note that `validate = false` on an input is different from `validate_json_inputs_by_default = false` on the template. The per-input flag prevents the validator from being compiled entirely, while the template-level flag only disables the validation check at runtime and can be toggled by integrations.
