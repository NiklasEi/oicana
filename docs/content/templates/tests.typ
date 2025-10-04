#import "/src/boxes.typ": *
#import "/src/code.typ": *


<testing>Oicana comes with test infrastructure for templates. To get started, create a directory called `tests` in a template directory.
Here is an example test collection `tests.toml` defining two snapshot tests:

#code("tests.toml", ```toml
tests_version = 1

[[test]]
name = "with_logo"

[[test.inputs]]
type = "blob"
key = "logo"
file = "../logo.jpg"

[[test.inputs]]
type = "json"
key = "data"
file = "data.json"
```)

\
All paths in a test collection are relative to its toml file. The collection above defines a single test with a `blob` input and a `json` input given as `logo.jpg` in the parent directory and `data.json` next to the test collection.
Executing `oicana test` for this template, will compile it with those inputs and attempt to compare the output with a `with_logo.png` living next to the test collection.

\
The tests directory will be recursively searched for any test collection files in the form of `<optional-prefix.>tests.toml`.

\
#example[The example templates #link("https://github.com/oicana/oicana-example-templates/tree/main/templates/table/tests")[`table`] and #link("https://github.com/oicana/oicana-example-templates/tree/main/templates/invoice/tests")[`invoice`] both define some simple snapshot tests.]

== Json input fuzzing

If you define and configure a schema for a json input, you can let Oicana fuzz that input as part of a snapshot test.

#code("tests.toml", ```toml
tests_version = 1

[[test]]
name = "fuzz_json_input"
snapshot = false

[[test.inputs]]
type = "json"
key = "data"
samples = 50
```)

Setting `snapshot = false` means no image files are created and compared. This is often the right choice for fuzzing tests, because the image output is likely expected to be different for different json input values. The configuration of `50` samples will cause Oicana to compile the template with `50` random values for the json input that all satisfy the schema. If the schema is very large, it might make sense to increase the number of samples.

== Full example configuration

A maximal and documented example test collection:

#code("tests.toml", ```toml
tests_version = 1

[[test]]
name = "with_logo" # Required
mode = "development" # Optional, default "production" - decides if `development` values of inputs get used or not
snapshot = "my_snapshot.png" # Optional, default "<test-name>.png" - relative path to a png file that will be compared to the test output

[[test.inputs]]
type = "blob" # Required - `blob` or `json`
key = "logo" # Required - key of input as configured in the template manifest under test
file = "../logo.jpg" # Required - relative path to a file that will be the value of this input
meta = { image_format = "jpg" } # Optional, default `none` - meta dictionary for the blob input (see input documentation)

[[test.inputs]]
type = "json"
key = "data"
file = "data.json"


[[test]]
name = "test_without_snapshot_comparison"
snapshot = false # this disables comparing the test output with a snapshot file


[[test]]
name = "fuzz_json_input"
snapshot = false

[[test.inputs]]
type = "json"
key = "data"
samples = 50 # this requires that the "data" input has a json schema configured in `typst.toml`


# Any number of additional tests in this collection
[[test]]
name = "a_second_test"
```)
